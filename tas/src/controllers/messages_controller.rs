use crate::app::{AppState, Message, WorkspaceMessage};
use crate::controllers::{
    pm_controller,
    bestiary_controller,
    universe_controller,
    locations_controller,
    timeline_controller,
    the_forge_controller,
};
use crate::state::ToastKind;

pub fn update(state: &mut AppState, message: Message) {
    match message {
        Message::Navigate(route) => state.route = route,
        Message::BackToUniverses => state.route = crate::app::Route::UniverseList,
        Message::BackToUniverse(id) => state.route = crate::app::Route::UniverseDetail { universe_id: id },
        Message::OpenTimeline(id) => state.route = crate::app::Route::Timeline { universe_id: id },

        Message::GoToLocation(universe_id, location_id) => {
            state.route = crate::app::Route::Locations { universe_id };
            state.selected_location = Some(location_id.clone());

            // Auto-expand tree to find the location
            let mut current_search = Some(location_id);
            let mut safeguard = 0;
            while let Some(curr_id) = current_search {
                if safeguard > 50 { break; } // Prevent infinite loops
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
            WorkspaceMessage::CreateStart => {
                state.is_creating_project = true;
                state.new_project_name.clear();
            }
            WorkspaceMessage::CreateCancel => state.is_creating_project = false,
            WorkspaceMessage::NameChanged(v) => state.new_project_name = v,
            _ => {}
        },

        Message::ProjectsLoaded(projs) => state.projects = projs,

        // Delegate module-specific messages (ZIP canonical names)
        Message::Pm(msg) => pm_controller::update(state, msg),
        Message::Bestiary(msg) => bestiary_controller::update(state, msg),
        Message::Universe(msg) => universe_controller::update(state, msg),
        Message::Locations(msg) => locations_controller::update(state, msg),
        Message::Timeline(msg) => timeline_controller::update(state, msg),
        Message::TheForge(msg) => the_forge_controller::update(state, msg),

        // Global Mouse Events (Delegated to PM Controller for Drag&Drop)
        Message::MouseMoved(p) => {
            pm_controller::handle_mouse_moved(state, p);
        }

        Message::MouseReleased => {
            pm_controller::handle_mouse_released(state);
        }

        // Results handling (ZIP canonical fields)
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

        // --- THE FORGE RESULTS ---
        Message::StoriesFetched(Ok(v)) => state.stories = v,
        Message::ScenesFetched(Ok(v)) => state.active_story_scenes = v,

        Message::SnapshotsFetched(Ok(v)) => state.snapshots = v,
        Message::SchemaVersionFetched(Ok(v)) => state.debug_schema_version = Some(v),
        Message::IntegrityFetched(Ok(v)) => {
            state.integrity_issues = v;
            state.integrity_busy = false;
        }

        // Errors (ZIP canonical grouping)
        Message::UniversesFetched(Err(e))
        | Message::BoardsFetched(Err(e))
        | Message::PmBoardFetched(Err(e))
        | Message::CreaturesFetched(Err(e))
        | Message::LocationsFetched(Err(e))
        | Message::TimelineFetched(Err(e))
        | Message::SnapshotsFetched(Err(e))
        | Message::SchemaVersionFetched(Err(e))
        | Message::IntegrityFetched(Err(e))
        | Message::StoriesFetched(Err(e))
        | Message::ScenesFetched(Err(e)) => {
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
