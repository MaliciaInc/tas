use crate::app::AppState;
use crate::state::{DbAction, DemoResetScope, ToastKind};

pub fn handle_action_done(state: &mut AppState, result: &Result<(), String>) {
    let inflight = state.db_inflight.clone();
    state.db_inflight = None;

    match result {
        Ok(_) => {
            // Force reload (preserve existing behavior exactly)
            state.data_dirty = true;
            state.loaded_creatures_universe = None;
            state.loaded_locations_universe = None;
            state.loaded_timeline_universe = None;
            state.loaded_snapshots_universe = None;
            state.pm_data = None;

            if let Some(action) = inflight {
                match action {
                    DbAction::ResetDemoDataScoped(_, scope) => {
                        let msg = match scope {
                            DemoResetScope::All => "Demo reset complete: Bestiary(7), Locations(7), Timeline(5 eras/15 events), PM Tools(6 cards)",
                            DemoResetScope::Timeline => "Timeline reset complete: 5 eras / 15 events",
                            DemoResetScope::Locations => "Locations reset complete: 7 locations",
                            DemoResetScope::Bestiary => "Bestiary reset complete: 7 creatures",
                            DemoResetScope::PmTools => "PM Tools reset complete: 6 cards",
                        };
                        state.show_toast(msg, ToastKind::Success);
                    }
                    DbAction::InjectDemoData(_) => {
                        state.show_toast("Demo data injected", ToastKind::Success);
                    }
                    _ => {}
                }
            }
        }
        Err(e) => {
            state.show_toast(format!("Action failed: {}", e), ToastKind::Error);
        }
    }
}
