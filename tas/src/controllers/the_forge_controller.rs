use iced::widget::text_editor;
use crate::app::AppState;
use crate::messages::TheForgeMessage;
use crate::state::{DbAction, ToastKind};

pub fn update(state: &mut AppState, message: TheForgeMessage) {
    match message {
        TheForgeMessage::Open(mut universe_id) => {
            state.route = crate::app::Route::Forge;

            // LÓGICA ROBUSTA: Si no se especifica ID, usar el cargado o Default Standalone.
            if universe_id.is_empty() {
                if let Some(current) = &state.loaded_forge_universe {
                    universe_id = current.clone();
                } else {
                    universe_id = "u-standalone".to_string();
                }
            }

            if state.loaded_forge_universe.as_ref() != Some(&universe_id) {
                state.loaded_forge_universe = Some(universe_id.clone());
                state.stories.clear();
                state.active_story_id = None;
                state.active_story_scenes.clear();
                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
            }
        }

        // CAMBIO DE CONTEXTO DESDE UI
        TheForgeMessage::UniverseChanged(new_id) => {
            if state.loaded_forge_universe.as_ref() != Some(&new_id) {
                state.loaded_forge_universe = Some(new_id);
                // Limpiamos todo para forzar recarga limpia
                state.stories.clear();
                state.active_story_id = None;
                state.active_story_scenes.clear();
                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
            }
        }

        TheForgeMessage::CreateStory => {
            if let Some(uid) = &state.loaded_forge_universe {
                state.queue(DbAction::CreateStory(uid.clone(), "New Story".to_string()));
            }
        }

        TheForgeMessage::DeleteStory(id) => {
            state.queue(DbAction::DeleteStory(id.clone()));
            if state.active_story_id.as_deref() == Some(&id) {
                state.active_story_id = None;
                state.active_story_scenes.clear();
                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
            }
        }

        TheForgeMessage::SelectStory(id) => {
            state.active_story_id = Some(id);
            state.active_scene_id = None;
            state.forge_content = text_editor::Content::new();
        }

        TheForgeMessage::CreateScene => {
            if let Some(sid) = &state.active_story_id {
                state.queue(DbAction::CreateScene(sid.clone(), "New Scene".to_string()));
            } else {
                state.show_toast("Select a Story first", ToastKind::Error);
            }
        }

        TheForgeMessage::DeleteScene(id) => {
            state.queue(DbAction::DeleteScene(id.clone()));
            if state.active_scene_id.as_deref() == Some(&id) {
                state.active_scene_id = None;
                state.forge_content = text_editor::Content::new();
            }
        }

        TheForgeMessage::SelectScene(id) => {
            state.active_scene_id = Some(id.clone());
            if let Some(scene) = state.active_story_scenes.iter().find(|s| s.id == id) {
                state.forge_content = text_editor::Content::with_text(&scene.body);
            }
        }

        TheForgeMessage::StoryTitleChanged(new_title) => {
            if let Some(sid) = &state.active_story_id {
                if let Some(idx) = state.stories.iter().position(|s| s.id == *sid) {
                    state.stories[idx].title = new_title;
                    let story_copy = state.stories[idx].clone();
                    state.queue(DbAction::UpdateStory(story_copy));
                }
            }
        }

        TheForgeMessage::SceneTitleChanged(new_title) => {
            if let Some(sid) = &state.active_scene_id {
                if let Some(idx) = state.active_story_scenes.iter().position(|s| s.id == *sid) {
                    state.active_story_scenes[idx].title = new_title;
                    let scene_copy = state.active_story_scenes[idx].clone();
                    state.queue(DbAction::UpdateScene(scene_copy));
                }
            }
        }

        TheForgeMessage::SceneBodyChanged(action) => {
            state.forge_content.perform(action);
            state.data_dirty = true;
        }

        TheForgeMessage::SaveCurrentScene => {
            if let Some(sid) = &state.active_scene_id {
                if let Some(idx) = state.active_story_scenes.iter().position(|s| s.id == *sid) {
                    let current_text = state.forge_content.text();
                    state.active_story_scenes[idx].body = current_text.clone();
                    let scene_copy = state.active_story_scenes[idx].clone();
                    state.queue(DbAction::UpdateScene(scene_copy));
                    state.show_toast("Scene saved", ToastKind::Success);
                }
            }
        }
    }
}