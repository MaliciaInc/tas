use iced::widget::text_editor;
use iced::Point;
use std::time::Instant;
use uuid::Uuid;
use crate::app::{AppState, PmMessage, PmState};
use crate::model::Card;
use crate::state::{DbAction, ToastKind};

pub fn update(state: &mut AppState, message: PmMessage) {
    match message {
        PmMessage::BoardNameChanged(name) => state.new_board_name = name,

        PmMessage::CreateBoard => {
            if !state.new_board_name.trim().is_empty() {
                state.queue(DbAction::CreateBoard(state.new_board_name.clone()));
                state.new_board_name.clear();
                state.show_toast("Creating board...", ToastKind::Info);
            }
        },
        PmMessage::DeleteBoard(id) => state.queue(DbAction::DeleteBoard(id)),
        PmMessage::OpenBoard(id) => state.route = crate::app::Route::PmBoard { board_id: id },
        PmMessage::BoardLoaded(data) => state.pm_data = Some(data),

        PmMessage::DragStart(c) => {
            let now = Instant::now();
            let mut is_double_click = false;
            if let Some((last_id, last_time)) = &state.last_pm_click {
                if *last_id == c.id && now.duration_since(*last_time).as_millis() < 500 { is_double_click = true; }
            }
            state.last_pm_click = Some((c.id.clone(), now));

            if is_double_click {
                state.pm_state = PmState::Editing {
                    card_id: Some(c.id),
                    column_id: c.column_id,
                    title: c.title,
                    description: text_editor::Content::with_text(&c.description),
                    priority: c.priority
                };
            } else {
                state.pm_state = PmState::Dragging {
                    original_col: c.column_id.clone(),
                    card: c,
                    drag_start: Point::new(0.0, 0.0),
                    current_cursor: Point::new(0.0, 0.0),
                    active: false
                };
            }
        }

        PmMessage::ColumnHovered(cid) => state.hovered_column = Some(cid),
        PmMessage::CardHovered(cid) => state.hovered_card = Some(cid),

        PmMessage::OpenCreate(cid) => {
            state.pm_state = PmState::Editing {
                card_id: None,
                column_id: cid,
                title: String::new(),
                description: text_editor::Content::new(),
                priority: "Medium".to_string()
            }
        },

        PmMessage::OpenGlobalCreate => {
            let mut target_col_id = String::new();
            if let Some(data) = &state.pm_data {
                // 1. Try to find canonical "To Do" column by ID (more robust)
                if let Some((col, _)) = data.columns.iter().find(|(c, _)| c.id == "col-todo") {
                    target_col_id = col.id.clone();
                }
                // 2. Fallback to name search
                else if let Some((col, _)) = data.columns.iter().find(|(c, _)| c.name.trim().eq_ignore_ascii_case("to do")) {
                    target_col_id = col.id.clone();
                }
                // 3. Fallback to first column
                else if !data.columns.is_empty() {
                    target_col_id = data.columns[0].0.id.clone();
                }
            }

            if !target_col_id.is_empty() {
                state.pm_state = PmState::Editing {
                    card_id: None,
                    column_id: target_col_id,
                    title: String::new(),
                    description: text_editor::Content::new(),
                    priority: "Medium".to_string()
                };
            } else {
                state.show_toast("No columns available to create task", ToastKind::Error);
            }
        },

        PmMessage::OpenEdit(c) => {
            state.pm_state = PmState::Editing {
                card_id: Some(c.id),
                column_id: c.column_id,
                title: c.title,
                description: text_editor::Content::with_text(&c.description),
                priority: c.priority
            }
        },

        PmMessage::TitleChanged(v) => if let PmState::Editing { title, .. } = &mut state.pm_state { *title = v },
        PmMessage::DescChanged(action) => if let PmState::Editing { description, .. } = &mut state.pm_state { description.perform(action); },
        PmMessage::PriorityChanged(v) => if let PmState::Editing { priority, .. } = &mut state.pm_state { *priority = v },

        PmMessage::Cancel => state.pm_state = PmState::Idle,

        PmMessage::Save => {
            if let PmState::Editing { card_id, column_id, title, description, priority } = &state.pm_state {
                if !title.trim().is_empty() && !column_id.is_empty() {
                    let card = Card {
                        id: card_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()),
                        column_id: column_id.clone(),
                        title: title.clone(),
                        description: description.text(),
                        position: 10000.0, // Backend will fix position if needed, or DragNDrop handles reorder
                        priority: priority.clone()
                    };
                    state.queue(DbAction::SaveCard(card));
                    state.show_toast("Task saved", ToastKind::Success);
                }
            }
            state.pm_state = PmState::Idle;
        },

        PmMessage::Delete => {
            if let PmState::Editing { card_id: Some(id), .. } = &state.pm_state {
                state.queue(DbAction::DeleteCard(id.clone()));
            }
            state.pm_state = PmState::Idle;
        }
    }
}

// --- PUBLIC HELPERS FOR ROOT CONTROLLER (Global Mouse Events) ---

pub fn handle_mouse_moved(state: &mut AppState, p: Point) {
    if let PmState::Dragging { current_cursor, drag_start, active, .. } = &mut state.pm_state {
        *current_cursor = p;
        if !*active {
            let dx = p.x - drag_start.x;
            let dy = p.y - drag_start.y;
            // Only start dragging after moving 10 pixels to prevent accidental clicks
            if (dx * dx + dy * dy).sqrt() > 10.0 {
                *active = true;
            }
        }
    }
}

pub fn handle_mouse_released(state: &mut AppState) {
    // 1. COLLECT PHASE: Calculate actions without mutating state directly yet
    let mut actions_to_queue = Vec::new();

    // Create a scope to restrict the immutable borrow of state
    {
        if let PmState::Dragging { card, active, .. } = &state.pm_state {
            if *active {
                if let Some(target_col) = &state.hovered_column {
                    if let Some(data) = &state.pm_data {
                        if let Some((_, cards)) = data.columns.iter().find(|(col, _)| col.id == *target_col) {
                            let mut new_pos = 0.0;
                            let mut found_neighbor = false;
                            let mut needs_rebalance = false;

                            if let Some(hover_id) = &state.hovered_card {
                                if let Some(idx) = cards.iter().position(|c| c.id == *hover_id) {
                                    let neighbor_pos = cards[idx].position;
                                    if idx > 0 {
                                        let prev_pos = cards[idx - 1].position;
                                        new_pos = (prev_pos + neighbor_pos) / 2.0;

                                        // Check precision health
                                        if (neighbor_pos - prev_pos).abs() < 0.1 {
                                            needs_rebalance = true;
                                        }
                                    } else {
                                        // Insert at very top
                                        new_pos = neighbor_pos / 2.0;
                                        if neighbor_pos < 0.1 { needs_rebalance = true; }
                                    }
                                    found_neighbor = true;
                                }
                            }

                            if !found_neighbor {
                                // Append to bottom
                                new_pos = if let Some(last) = cards.last() { last.position + 1000.0 } else { 1000.0 };
                            }

                            // Add to list instead of queueing immediately
                            actions_to_queue.push(DbAction::MoveCard(card.id.clone(), target_col.clone(), new_pos));

                            if needs_rebalance {
                                actions_to_queue.push(DbAction::RebalanceColumn(target_col.clone()));
                            }
                        }
                    }
                }
            }
        }
    } // Immutable borrows end here

    // 2. APPLY PHASE: Safe to mutate state now
    if !actions_to_queue.is_empty() {
        for action in actions_to_queue {
            if let DbAction::RebalanceColumn(_) = action {
                crate::logger::info("Detected low float precision. Queueing rebalance.");
            }
            state.queue(action);
        }
    }

    // Always reset state on release
    state.pm_state = PmState::Idle;
    state.hovered_column = None;
    state.hovered_card = None;
}