use iced::Task;

use crate::app::{AppState, Message, PmState, WorkspaceMessage};
use crate::db::Database;
use crate::state::{DbAction, ToastKind};
use crate::controllers::{pm_controller, bestiary_controller, universe_controller, locations_controller, timeline_controller};
use crate::project_manager::ProjectManager;
use crate::Message as MainMessage;

pub fn update(state: &mut AppState, message: Message) {
    match message {
        Message::Navigate(route) => state.route = route,
        Message::BackToUniverses => state.route = crate::app::Route::UniverseList,
        Message::BackToUniverse(id) => state.route = crate::app::Route::UniverseDetail { universe_id: id },
        Message::OpenTimeline(id) => state.route = crate::app::Route::Timeline { universe_id: id },

        Message::GoToLocation(universe_id, location_id) => {
            state.route = crate::app::Route::Locations { universe_id };
            state.selected_location = Some(location_id.clone());

            let mut current_search = Some(location_id);
            let mut safeguard = 0;
            while let Some(curr_id) = current_search {
                if safeguard > 50 { break; }
                safeguard += 1;

                if let Some(loc) = state.locations.iter().find(|l| l.id == curr_id) {
                    if let Some(parent_id) = &loc.parent_id {
                        state.expanded_locations.insert(parent_id.clone());
                        current_search = Some(parent_id.clone());
                    } else {
                        current_search = None;
                    }
                } else {
                    current_search = None;
                }
            }
        }

        Message::Workspace(msg) => match msg {
            WorkspaceMessage::CreateStart => { state.is_creating_project = true; state.new_project_name.clear(); }
            WorkspaceMessage::CreateCancel => state.is_creating_project = false,
            WorkspaceMessage::NameChanged(v) => state.new_project_name = v,
            _ => {}
        },

        Message::ProjectsLoaded(projs) => state.projects = projs,

        Message::Pm(msg) => pm_controller::update(state, msg),
        Message::Bestiary(msg) => bestiary_controller::update(state, msg),
        Message::Universe(msg) => universe_controller::update(state, msg),
        Message::Locations(msg) => locations_controller::update(state, msg),
        Message::Timeline(msg) => timeline_controller::update(state, msg),

        Message::MouseMoved(p) => {
            if let PmState::Dragging { current_cursor, drag_start, active, .. } = &mut state.pm_state {
                *current_cursor = p;
                if !*active {
                    let dx = p.x - drag_start.x;
                    let dy = p.y - drag_start.y;
                    if (dx * dx + dy * dy).sqrt() > 10.0 {
                        *active = true;
                    }
                }
            }
        }

        Message::MouseReleased => {
            if let PmState::Dragging { card, active, .. } = &state.pm_state {
                if *active {
                    if let Some(target_col) = &state.hovered_column {
                        if let Some(data) = &state.pm_data {
                            if let Some((_, cards)) = data.columns.iter().find(|(col, _)| col.id == *target_col) {
                                let mut new_pos = 0.0;
                                let mut found_neighbor = false;

                                if let Some(hover_id) = &state.hovered_card {
                                    if let Some(idx) = cards.iter().position(|c| c.id == *hover_id) {
                                        let neighbor_pos = cards[idx].position;
                                        if idx > 0 {
                                            new_pos = (cards[idx - 1].position + neighbor_pos) / 2.0;
                                        } else {
                                            new_pos = neighbor_pos / 2.0;
                                        }
                                        found_neighbor = true;
                                    }
                                }

                                if !found_neighbor {
                                    new_pos = if let Some(last) = cards.last() { last.position + 1000.0 } else { 1000.0 };
                                }

                                state.queue(DbAction::MoveCard(card.id.clone(), target_col.clone(), new_pos));
                            }
                        }
                    }
                }
            }

            state.pm_state = PmState::Idle;
            state.hovered_column = None;
            state.hovered_card = None;
        }

        // Results
        Message::UniversesFetched(Ok(v)) => state.universes = v,
        Message::BoardsFetched(Ok(v)) => state.boards_list = v,
        Message::PmBoardFetched(Ok(v)) => state.pm_data = Some(v),

        Message::CreaturesFetched(Ok(v)) => {
            state.creatures = v;
            if let crate::app::Route::Bestiary { universe_id } = &state.route {
                state.loaded_creatures_universe = Some(universe_id.clone());
            }
        }

        Message::LocationsFetched(Ok(v)) => {
            state.locations = v;
            if let crate::app::Route::Locations { universe_id } = &state.route {
                state.loaded_locations_universe = Some(universe_id.clone());
            }
        }

        Message::TimelineFetched(Ok((events, eras))) => {
            state.timeline_events = events;
            state.timeline_eras = eras;
            if let crate::app::Route::Timeline { universe_id } = &state.route {
                state.loaded_timeline_universe = Some(universe_id.clone());
            }
        }

        Message::SnapshotsFetched(Ok(v)) => state.snapshots = v,
        Message::SchemaVersionFetched(Ok(v)) => state.debug_schema_version = Some(v),
        Message::IntegrityFetched(Ok(v)) => {
            state.integrity_issues = v;
            state.integrity_busy = false;
        }

        // Errors
        Message::UniversesFetched(Err(e))
        | Message::BoardsFetched(Err(e))
        | Message::PmBoardFetched(Err(e))
        | Message::CreaturesFetched(Err(e))
        | Message::LocationsFetched(Err(e))
        | Message::TimelineFetched(Err(e))
        | Message::SnapshotsFetched(Err(e))
        | Message::SchemaVersionFetched(Err(e))
        | Message::IntegrityFetched(Err(e)) => {
            state.show_toast(format!("Error loading data: {}", e), ToastKind::Error);
            state.integrity_busy = false;
        }

        Message::Tick(_) => {
            let now = std::time::Instant::now();
            state.toasts.retain(|t| now.duration_since(t.created_at).as_secs() < t.ttl_secs);
        }

        Message::ToastDismiss(id) => state.toasts.retain(|t| t.id != id),

        _ => {}
    }
}

pub fn post_event_tasks(state: &mut AppState, db: &Option<Database>) -> Vec<Task<MainMessage>> {
    let mut tasks = Vec::new();
    let Some(db_base) = db else { return tasks };

    // Keep projects fresh in launcher-ish contexts
    if state.projects.is_empty() {
        tasks.push(Task::perform(async { ProjectManager::load_projects() }, MainMessage::ProjectsLoaded));
    }

    // 1) Process DB queue (ONE at a time)
    if state.db_inflight.is_none() {
        if let Some(action) = state.db_queue.pop_front() {
            state.db_inflight = Some(action.clone());
            let db = db_base.clone();

            tasks.push(Task::perform(
                async move {
                    match action {
                        DbAction::CreateUniverse(n, d) => db.create_universe(n, d).await.map_err(|e| e.to_string()),
                        DbAction::DeleteUniverse(id) => db.delete_universe(id).await.map_err(|e| e.to_string()),
                        DbAction::InjectDemoData(id) => db.inject_demo_data(id).await.map_err(|e| e.to_string()),
                        DbAction::ResetDemoDataScoped(id, scope) => db.reset_demo_data_scoped(id, scope).await.map_err(|e| e.to_string()),

                        DbAction::SnapshotCreate { universe_id, name } => db.snapshot_create(universe_id, name).await.map_err(|e| e.to_string()),
                        DbAction::SnapshotDelete { snapshot_id } => db.snapshot_delete(snapshot_id).await.map_err(|e| e.to_string()),
                        DbAction::SnapshotRestore { snapshot_id } => db.snapshot_restore(snapshot_id).await.map_err(|e| e.to_string()),

                        DbAction::CreateBoard(n) => db.create_board(n).await.map_err(|e| e.to_string()),
                        DbAction::DeleteBoard(id) => db.delete_board(id).await.map_err(|e| e.to_string()),

                        DbAction::SaveCreature(c, uid) => db.upsert_creature(c, uid).await.map_err(|e| e.to_string()),
                        DbAction::ArchiveCreature(id, st) => db.set_creature_archived(id, st).await.map_err(|e| e.to_string()),
                        DbAction::DeleteCreature(id) => db.delete_creature(id).await.map_err(|e| e.to_string()),

                        DbAction::SaveLocation(l) => db.upsert_location(l).await.map_err(|e| e.to_string()),
                        DbAction::DeleteLocation(id) => db.delete_location(id).await.map_err(|e| e.to_string()),

                        DbAction::SaveEvent(e) => db.upsert_timeline_event(e).await.map_err(|e| e.to_string()),
                        DbAction::DeleteEvent(id) => db.delete_timeline_event(id).await.map_err(|e| e.to_string()),
                        DbAction::SaveEra(e) => db.upsert_timeline_era(e).await.map_err(|e| e.to_string()),
                        DbAction::DeleteEra(id) => db.delete_timeline_era(id).await.map_err(|e| e.to_string()),

                        DbAction::SaveCard(c) => db.upsert_card(c).await.map_err(|e| e.to_string()),
                        DbAction::MoveCard(cid, col, pos) => db.move_card(cid, col, pos).await.map_err(|e| e.to_string()),
                        DbAction::DeleteCard(id) => db.delete_card(id).await.map_err(|e| e.to_string()),
                    }
                },
                MainMessage::ActionDone,
            ));
        }
    }

    // 2) Lazy fetch for current route (this is what prevents "UI disconnected from DB")
    if state.db_inflight.is_none() {
        match &state.route {
            crate::app::Route::UniverseList => {
                let db = db_base.clone();
                tasks.push(Task::perform(async move { db.get_all_universes().await.map_err(|e| e.to_string()) }, MainMessage::UniversesFetched));
            }

            crate::app::Route::PmList => {
                let db = db_base.clone();
                tasks.push(Task::perform(async move { db.get_all_boards().await.map_err(|e| e.to_string()) }, MainMessage::BoardsFetched));
            }

            crate::app::Route::PmBoard { board_id } => {
                let need_fetch = state.pm_data.as_ref().map(|d| d.board.id != *board_id).unwrap_or(true);
                if need_fetch {
                    let db = db_base.clone();
                    let bid = board_id.clone();
                    tasks.push(Task::perform(async move { db.get_kanban_data(bid).await.map_err(|e| e.to_string()) }, MainMessage::PmBoardFetched));
                }
            }

            crate::app::Route::Bestiary { universe_id } => {
                if state.loaded_creatures_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(async move { db.get_creatures(uid).await.map_err(|e| e.to_string()) }, MainMessage::CreaturesFetched));
                }
                // locations are needed for dropdowns
                let db = db_base.clone();
                let uid = universe_id.clone();
                tasks.push(Task::perform(async move { db.get_locations_flat(uid).await.map_err(|e| e.to_string()) }, MainMessage::LocationsFetched));
            }

            crate::app::Route::Locations { universe_id } => {
                if state.loaded_locations_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(async move { db.get_locations_flat(uid).await.map_err(|e| e.to_string()) }, MainMessage::LocationsFetched));
                }
            }

            crate::app::Route::Timeline { universe_id } => {
                if state.loaded_timeline_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(async move {
                        let events = db.get_timeline_events(uid.clone()).await.map_err(|e| e.to_string())?;
                        let eras = db.get_timeline_eras(uid).await.map_err(|e| e.to_string())?;
                        Ok((events, eras))
                    }, MainMessage::TimelineFetched));
                }
                // locations are needed for event editor dropdowns
                let db = db_base.clone();
                let uid = universe_id.clone();
                tasks.push(Task::perform(async move { db.get_locations_flat(uid).await.map_err(|e| e.to_string()) }, MainMessage::LocationsFetched));
            }

            crate::app::Route::UniverseDetail { universe_id } => {
                // schema version when overlay open
                if state.debug_overlay_open && state.debug_schema_version.is_none() {
                    let db = db_base.clone();
                    tasks.push(Task::perform(async move { db.get_schema_version().await.map_err(|e| e.to_string()) }, MainMessage::SchemaVersionFetched));
                }

                // snapshots list (Arhelis-only UI anyway, but safe to fetch for any)
                if state.loaded_snapshots_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(async move { db.snapshot_list(uid).await.map_err(|e| e.to_string()) }, MainMessage::SnapshotsFetched));
                    state.loaded_snapshots_universe = Some(universe_id.clone());
                }

                // integrity validation (triggered by state.integrity_busy)
                if state.integrity_busy {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(async move { db.validate_universe(uid).await.map_err(|e| e.to_string()) }, MainMessage::IntegrityFetched));
                }
            }

            _ => {}
        }
    }

    tasks
}
