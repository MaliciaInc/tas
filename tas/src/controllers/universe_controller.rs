use crate::app::{AppState, UniverseMessage};
use crate::state::{DbAction, ToastKind};

pub fn update(state: &mut AppState, message: UniverseMessage) {
    match message {
        UniverseMessage::NameChanged(v) => state.new_universe_name = v,
        UniverseMessage::DescChanged(v) => state.new_universe_desc = v,

        UniverseMessage::Create => {
            if !state.new_universe_name.trim().is_empty() {
                state.queue(DbAction::CreateUniverse(
                    state.new_universe_name.clone(),
                    state.new_universe_desc.clone(),
                ));
                state.new_universe_name.clear();
                state.new_universe_desc.clear();
                state.show_toast("Creating universe...", ToastKind::Info);
            }
        }

        UniverseMessage::Delete(id) => {
            state.queue(DbAction::DeleteUniverse(id));
            state.show_toast("Universe deleted", ToastKind::Info);
        }

        UniverseMessage::Open(id) => {
            state.route = crate::app::Route::UniverseDetail { universe_id: id };
        }

        UniverseMessage::InjectDemoData(id) => {
            state.queue(DbAction::InjectDemoData(id));
            state.show_toast("Injecting demo data...", ToastKind::Info);
        }

        UniverseMessage::ResetDemoPrompt(uid, scope) => state.pending_demo_reset = Some((uid, scope)),
        UniverseMessage::ResetDemoCancel => state.pending_demo_reset = None,
        UniverseMessage::ResetDemoConfirm => {
            if let Some((uid, scope)) = state.pending_demo_reset.take() {
                state.queue(DbAction::ResetDemoDataScoped(uid, scope));
                state.show_toast("Resetting demo data...", ToastKind::Info);
            }
        }

        UniverseMessage::ToggleDeveloperPanel => {
            state.dev_panel_open = !state.dev_panel_open;
        }

        UniverseMessage::ToggleDebugOverlay => {
            state.debug_overlay_open = !state.debug_overlay_open;
            // Force refresh next render tick
            state.debug_schema_version = None;
        }

        UniverseMessage::SnapshotNameChanged(v) => state.snapshot_name = v,
        UniverseMessage::SnapshotCreate(universe_id) => {
            let name = state.snapshot_name.trim().to_string();
            if !name.is_empty() {
                state.queue(DbAction::SnapshotCreate { universe_id, name });
                state.snapshot_name.clear();
                state.show_toast("Creating snapshot...", ToastKind::Info);
            }
        }
        UniverseMessage::SnapshotRefresh(universe_id) => {
            state.loaded_snapshots_universe = None;
            state.show_toast("Refreshing snapshots...", ToastKind::Info);
            state.route = crate::app::Route::UniverseDetail { universe_id };
        }
        UniverseMessage::SnapshotRestore(snapshot_id) => {
            state.queue(DbAction::SnapshotRestore { snapshot_id });
            state.show_toast("Restoring snapshot...", ToastKind::Info);
        }
        UniverseMessage::SnapshotDelete(snapshot_id) => {
            state.queue(DbAction::SnapshotDelete { snapshot_id });
            state.show_toast("Deleting snapshot...", ToastKind::Info);
        }

        UniverseMessage::ValidateUniverse(_universe_id) => {
            // We will fetch issues via root_controller task (not queued) to avoid breaking inflight clearing.
            state.integrity_busy = true;
        }
    }
}
