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
            state.show_toast("Injecting demo data (please wait)...", ToastKind::Info);
        }

        UniverseMessage::ResetDemoPrompt(uid, scope) => {
            state.pending_demo_reset = Some((uid, scope));
        }

        UniverseMessage::ResetDemoCancel => {
            state.pending_demo_reset = None;
        }

        UniverseMessage::ResetDemoConfirm => {
            if let Some((uid, scope)) = state.pending_demo_reset.take() {
                state.queue(DbAction::ResetDemoDataScoped(uid, scope));
                state.show_toast("Resetting demo data...", ToastKind::Info);
            }
        }
    }
}
