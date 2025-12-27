use iced::{Element, Length};
use iced::widget::{button, container, row, column, text, text_input, text_editor, scrollable, Space, pick_list};
use crate::app::{AppState, Message};
use crate::messages::TheForgeMessage;
use crate::ui::{self, Tokens};
use crate::model::Universe;

fn divider(t: Tokens) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| ui::container_style(t.border, t.border))
        .into()
}

pub fn the_forge_view<'a>(state: &'a AppState, t: Tokens) -> Element<'a, Message> {

    // 1. OBTENER UNIVERSO ACTIVO
    let active_universe_id = if let Some(uid) = &state.loaded_forge_universe {
        if uid.is_empty() { "u-standalone".to_string() } else { uid.clone() }
    } else {
        "u-standalone".to_string()
    };

    // 2. PREPARAR DROPDOWN
    let universes: Vec<Universe> = state.universes.clone();

    // Si estamos en Standalone, fabricamos el objeto visual para el selector
    let selected_universe = if active_universe_id == "u-standalone" {
        Some(Universe {
            id: "u-standalone".to_string(),
            name: "Standalone Stories".to_string(),
            description: "".to_string(),
            archived: false
        })
    } else {
        universes.iter().find(|u| u.id == active_universe_id).cloned()
    };

    let mut pick_options = universes.clone();
    // Inyección manual de "Standalone" solo para este menú
    if !pick_options.iter().any(|u| u.id == "u-standalone") {
        pick_options.insert(0, Universe {
            id: "u-standalone".to_string(),
            name: "Standalone Stories".to_string(),
            description: "".to_string(),
            archived: false
        });
    }

    let universe_selector = row![
        text("Context:").size(12).color(t.muted_fg),
        pick_list(
            pick_options,
            selected_universe,
            |u| Message::TheForge(TheForgeMessage::UniverseChanged(u.id))
        )
        .text_size(12)
        .width(Length::Fill)
        .style(move |_, _| pick_list::Style {
            text_color: t.foreground,
            placeholder_color: t.muted_fg,
            handle_color: t.muted_fg,
            background: t.input_border.into(),
            border: iced::Border { color: t.border, width: 1.0, radius: 4.0.into() },
        })
    ]
        .spacing(8)
        .align_y(iced::Alignment::Center)
        .padding(10);


    // --- LISTA DE HISTORIAS ---
    let stories_header = row![
        text("Stories").size(14).color(t.muted_fg).width(Length::Fill),
        button(text("+").size(16))
            .style(ui::ghost_button_style(t))
            .on_press(Message::TheForge(TheForgeMessage::CreateStory))
    ]
        .padding(10)
        .align_y(iced::Alignment::Center);

    let mut stories_list = column![].spacing(2);
    for story in &state.stories {
        let is_active = state.active_story_id.as_ref() == Some(&story.id);
        let bg = if is_active { t.active_bg } else { iced::Color::TRANSPARENT };

        let btn = button(
            column![
                    text(&story.title).size(14).color(if is_active { t.foreground } else { t.muted_fg }),
                    text(&story.status).size(10).color(ui::alpha(t.muted_fg, 0.5))
                ].spacing(2)
        )
            .width(Length::Fill)
            .padding(10)
            .style(move |_, _| {
                let mut s = button::Style::default();
                s.background = Some(bg.into());
                s.text_color = t.foreground;
                s
            })
            .on_press(Message::TheForge(TheForgeMessage::SelectStory(story.id.clone())));

        stories_list = stories_list.push(btn);
    }

    let col_stories = container(
        column![
            universe_selector,
            divider(t),
            stories_header,
            divider(t),
            scrollable(stories_list)
        ]
    )
        .width(Length::Fixed(240.0))
        .height(Length::Fill)
        .style(move |_| {
            let mut s = ui::container_style(t.shell_a, t.foreground);
            s.border.width = 1.0;
            s.border.color = ui::alpha(t.border, 0.5);
            s
        });

    // --- LISTA DE ESCENAS ---
    let scenes_header = row![
        text("Scenes").size(14).color(t.muted_fg).width(Length::Fill),
        button(text("+").size(16))
            .style(ui::ghost_button_style(t))
            .on_press(Message::TheForge(TheForgeMessage::CreateScene))
    ]
        .padding(10)
        .align_y(iced::Alignment::Center);

    let mut scenes_list = column![].spacing(2);

    if state.active_story_id.is_some() {
        if state.active_story_scenes.is_empty() {
            scenes_list = scenes_list.push(
                container(text("No scenes yet.").size(12).color(t.muted_fg)).padding(10)
            );
        } else {
            for scene in &state.active_story_scenes {
                let is_active = state.active_scene_id.as_ref() == Some(&scene.id);
                let bg = if is_active { t.active_bg } else { iced::Color::TRANSPARENT };

                let btn = button(
                    row![
                            text(format!("{}.", scene.position)).size(12).color(t.muted_fg).width(Length::Fixed(20.0)),
                            text(&scene.title).size(13).color(if is_active { t.foreground } else { t.muted_fg }).width(Length::Fill),
                            text(format!("{}w", scene.word_count)).size(10).color(ui::alpha(t.muted_fg, 0.5))
                        ]
                        .align_y(iced::Alignment::Center)
                )
                    .width(Length::Fill)
                    .padding(8)
                    .style(move |_, _| {
                        let mut s = button::Style::default();
                        s.background = Some(bg.into());
                        s.text_color = t.foreground;
                        s
                    })
                    .on_press(Message::TheForge(TheForgeMessage::SelectScene(scene.id.clone())));

                scenes_list = scenes_list.push(btn);
            }
        }
    } else {
        scenes_list = scenes_list.push(
            container(text("Select a story.").size(12).color(t.muted_fg)).padding(10)
        );
    }

    let col_scenes = container(
        column![
            scenes_header,
            divider(t),
            scrollable(scenes_list)
        ]
    )
        .width(Length::Fixed(220.0))
        .height(Length::Fill)
        .style(move |_| {
            let mut s = ui::container_style(ui::alpha(t.shell_b, 0.5), t.foreground);
            s.border.width = 1.0;
            s.border.color = ui::alpha(t.border, 0.5);
            s
        });


    // --- EDITOR AREA ---
    let editor_area: Element<Message> = if let Some(scene_id) = &state.active_scene_id {
        let scene_meta = state.active_story_scenes.iter().find(|s| s.id == *scene_id);
        let title_val = scene_meta.map(|s| s.title.clone()).unwrap_or_default();

        let toolbar = row![
            text_input("Scene Title", &title_val)
                .on_input(|s| Message::TheForge(TheForgeMessage::SceneTitleChanged(s)))
                .padding(6)
                .width(Length::Fill)
                .style(ui::input_style(t)),

            Space::new().width(10),

            ui::primary_button(t, "Save".into(), Message::TheForge(TheForgeMessage::SaveCurrentScene)),
            ui::danger_button(t, "Delete".into(), Message::TheForge(TheForgeMessage::DeleteScene(scene_id.clone())))
        ]
            .padding(10)
            .spacing(10);

        let editor = text_editor(&state.forge_content)
            .on_action(|a| Message::TheForge(TheForgeMessage::SceneBodyChanged(a)))
            .height(Length::Fill)
            .style(ui::text_editor_style(t));

        column![
            toolbar,
            container(editor).padding(20).width(Length::Fill).height(Length::Fill)
        ]
            .into()

    } else if state.active_story_id.is_some() {
        container(text("Select a scene to edit.").size(18).color(t.muted_fg))
            .width(Length::Fill).height(Length::Fill).center_x(Length::Fill).center_y(Length::Fill).into()
    } else {
        container(text("").size(18))
            .width(Length::Fill).height(Length::Fill).into()
    };

    let col_editor = container(editor_area)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| ui::container_style(t.background, t.foreground));

    row![
        col_stories,
        col_scenes,
        col_editor
    ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}