use iced::Task;

use crate::app::{AppState, Message};
use crate::controllers::db_controller;
use crate::db::Database;
use crate::project_manager::ProjectManager;

pub fn post_event_tasks(state: &mut AppState, db: &Option<Database>) -> Vec<Task<Message>> {
    let mut tasks = Vec::new();
    let Some(db_base) = db else { return tasks };

    // Keep projects fresh in launcher-ish contexts
    if state.projects.is_empty() {
        tasks.push(Task::perform(async { ProjectManager::load_projects() }, Message::ProjectsLoaded));
    }

    // 1) Process DB queue (ONE at a time)
    if state.db_inflight.is_none() {
        if let Some(action) = state.db_queue.pop_front() {
            state.db_inflight = Some(action.clone());
            let db = db_base.clone();

            // Delegate execution to db_controller (single source of truth)
            tasks.push(db_controller::task_execute(db, action));
        }
    }

    // 2) Lazy fetch for current route (Smart Polling)
    if state.db_inflight.is_none() {
        match &state.route {
            crate::app::Route::UniverseList => {
                let db = db_base.clone();
                tasks.push(Task::perform(
                    async move { db.get_all_universes().await.map_err(|e| e.to_string()) },
                    Message::UniversesFetched,
                ));
            }

            crate::app::Route::PmList => {
                let db = db_base.clone();
                tasks.push(Task::perform(
                    async move { db.get_all_boards().await.map_err(|e| e.to_string()) },
                    Message::BoardsFetched,
                ));
            }

            crate::app::Route::PmBoard { board_id } => {
                let need_fetch = state.pm_data.as_ref().map(|d| d.board.id != *board_id).unwrap_or(true);
                if need_fetch {
                    let db = db_base.clone();
                    let bid = board_id.clone();
                    tasks.push(Task::perform(
                        async move { db.get_kanban_data(bid).await.map_err(|e| e.to_string()) },
                        Message::PmBoardFetched,
                    ));
                }
            }

            crate::app::Route::Bestiary { universe_id } => {
                if state.loaded_creatures_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(
                        async move { db.get_creatures(uid).await.map_err(|e| e.to_string()) },
                        Message::CreaturesFetched,
                    ));
                }

                // Locations needed for dropdowns
                let db = db_base.clone();
                let uid = universe_id.clone();
                tasks.push(Task::perform(
                    async move { db.get_locations_flat(uid).await.map_err(|e| e.to_string()) },
                    Message::LocationsFetched,
                ));
            }

            crate::app::Route::Locations { universe_id } => {
                if state.loaded_locations_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(
                        async move { db.get_locations_flat(uid).await.map_err(|e| e.to_string()) },
                        Message::LocationsFetched,
                    ));
                }
            }

            crate::app::Route::Timeline { universe_id } => {
                if state.loaded_timeline_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(
                        async move {
                            let events = db.get_timeline_events(uid.clone()).await.map_err(|e| e.to_string())?;
                            let eras = db.get_timeline_eras(uid).await.map_err(|e| e.to_string())?;
                            Ok((events, eras))
                        },
                        Message::TimelineFetched,
                    ));
                }

                // Locations needed for event editor dropdowns
                let db = db_base.clone();
                let uid = universe_id.clone();
                tasks.push(Task::perform(
                    async move { db.get_locations_flat(uid).await.map_err(|e| e.to_string()) },
                    Message::LocationsFetched,
                ));
            }

            // --- THE FORGE FETCHING ---
            crate::app::Route::Forge => {
                if let Some(uid) = &state.loaded_forge_universe {
                    let db = db_base.clone();
                    let u = uid.clone();
                    tasks.push(Task::perform(
                        async move { db.get_stories(u).await.map_err(|e| e.to_string()) },
                        Message::StoriesFetched,
                    ));

                    if let Some(sid) = &state.active_story_id {
                        let db2 = db_base.clone();
                        let s = sid.clone();
                        tasks.push(Task::perform(
                            async move { db2.get_scenes(s).await.map_err(|e| e.to_string()) },
                            Message::ScenesFetched,
                        ));
                    }
                }
            }

            crate::app::Route::UniverseDetail { universe_id } => {
                // schema version check
                if state.debug_overlay_open && state.debug_schema_version.is_none() {
                    let db = db_base.clone();
                    tasks.push(Task::perform(
                        async move { db.get_schema_version().await.map_err(|e| e.to_string()) },
                        Message::SchemaVersionFetched,
                    ));
                }

                // snapshot list fetching
                if state.loaded_snapshots_universe.as_ref() != Some(universe_id) {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(
                        async move { db.snapshot_list(uid).await.map_err(|e| e.to_string()) },
                        Message::SnapshotsFetched,
                    ));
                    state.loaded_snapshots_universe = Some(universe_id.clone());
                }

                // integrity check trigger
                if state.integrity_busy {
                    let db = db_base.clone();
                    let uid = universe_id.clone();
                    tasks.push(Task::perform(
                        async move { db.validate_universe(uid).await.map_err(|e| e.to_string()) },
                        Message::IntegrityFetched,
                    ));
                }

                // Pre-load Forge Universe context
                state.loaded_forge_universe = Some(universe_id.clone());
            }

            _ => {}
        }
    }

    tasks
}
