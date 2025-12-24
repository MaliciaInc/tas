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
                state.pm_state = PmState::Editing { card_id: Some(c.id), column_id: c.column_id, title: c.title, description: text_editor::Content::with_text(&c.description), priority: c.priority };
            } else {
                state.pm_state = PmState::Dragging { original_col: c.column_id.clone(), card: c, drag_start: Point::new(0.0,0.0), current_cursor: Point::new(0.0,0.0), active: false };
            }
        }

        PmMessage::ColumnHovered(cid) => state.hovered_column = Some(cid),
        PmMessage::CardHovered(cid) => state.hovered_card = Some(cid),

        PmMessage::OpenCreate(cid) => {
            state.pm_state = PmState::Editing { card_id: None, column_id: cid, title: String::new(), description: text_editor::Content::new(), priority: "Medium".to_string() }
        },

        PmMessage::OpenGlobalCreate => {
            let mut target_col_id = String::new();
            if let Some(data) = &state.pm_data {
                for (col, _) in &data.columns { if col.name.trim().eq_ignore_ascii_case("to do") { target_col_id = col.id.clone(); break; } }
                if target_col_id.is_empty() && !data.columns.is_empty() { target_col_id = data.columns[0].0.id.clone(); }
            }
            if !target_col_id.is_empty() {
                state.pm_state = PmState::Editing { card_id: None, column_id: target_col_id, title: String::new(), description: text_editor::Content::new(), priority: "Medium".to_string() };
            }
        },

        PmMessage::OpenEdit(c) => {
            state.pm_state = PmState::Editing { card_id: Some(c.id), column_id: c.column_id, title: c.title, description: text_editor::Content::with_text(&c.description), priority: c.priority }
        },

        PmMessage::TitleChanged(v) => if let PmState::Editing { title, .. } = &mut state.pm_state { *title = v },
        PmMessage::DescChanged(action) => if let PmState::Editing { description, .. } = &mut state.pm_state { description.perform(action); },
        PmMessage::PriorityChanged(v) => if let PmState::Editing { priority, .. } = &mut state.pm_state { *priority = v },

        PmMessage::Cancel => state.pm_state = PmState::Idle,

        PmMessage::Save => {
            if let PmState::Editing { card_id, column_id, title, description, priority } = &state.pm_state {
                if !title.trim().is_empty() && !column_id.is_empty() {
                    let card = Card { id: card_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string()), column_id: column_id.clone(), title: title.clone(), description: description.text(), position: 10000.0, priority: priority.clone() };
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