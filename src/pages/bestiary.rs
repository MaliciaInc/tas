use iced::{Alignment, Color, Length};
use iced::widget::{container, mouse_area, text, text_input, Column, Row};

use crate::app::{AppState, Message};
use crate::model::Creature;
use crate::{pages::E, ui};

pub fn bestiary<'a>(state: &'a AppState, t: ui::Tokens, universe_id: &'a str) -> E<'a> {
    let universe_name = state
        .universes
        .iter()
        .find(|u| u.id == universe_id)
        .map(|u| u.name.as_str())
        .unwrap_or(universe_id);

    let header_left = Column::new()
        .spacing(4)
        .push(text(format!("Bestiary — {}", universe_name)).size(26).color(t.foreground))
        .push(
            text("Creatures, entities and beings that inhabit this universe.")
                .size(12)
                .color(t.muted_fg),
        );

    let header_right = Row::new()
        .spacing(10)
        .push(ui::outline_button(
            t,
            "Back to universe".to_string(),
            Message::BackToUniverse(universe_id.to_string()),
        ))
        .push(ui::outline_button(
            t,
            "All universes".to_string(),
            Message::BackToUniverses,
        ))
        .push(ui::primary_button(
            t,
            "Create creature".to_string(),
            Message::CreatureEditorOpenCreate,
        ));

    let header = Row::new()
        .align_y(Alignment::Center)
        .push(container(header_left).width(Length::Fill))
        .push(header_right);

    let active_header = text(format!("Active creatures ({})", state.creatures.len()))
        .size(12)
        .color(t.muted_fg);

    let cards = creatures_grid(t, &state.creatures);

    let archived_header = text("Archived creatures (0)").size(12).color(t.muted_fg);

    let archived = ui::card(
        t,
        container(text("No archived creatures.").size(12).color(t.muted_fg))
            .width(Length::Fill)
            .padding([14, 14])
            .into(),
    );

    let mut body = Column::new()
        .spacing(14)
        .push(header)
        .width(Length::Fill);

    if let Some(editor) = &state.creature_editor {
        body = body.push(editor_panel(t, editor));
    }

    body = body
        .push(active_header)
        .push(cards)
        .push(archived_header)
        .push(archived);

    ui::page_padding(body.into())
}

fn editor_panel<'a>(t: ui::Tokens, editor: &'a crate::app::CreatureEditor) -> E<'a> {
    let title = if editor.index.is_some() {
        "Edit creature"
    } else {
        "Create creature"
    };

    let name_input = text_input("Name", &editor.name)
        .on_input(Message::CreatureEditorNameChanged)
        .padding(10);

    let kind_input = text_input("Kind / tags", &editor.kind)
        .on_input(Message::CreatureEditorKindChanged)
        .padding(10);

    let habitat_input = text_input("Habitat", &editor.habitat)
        .on_input(Message::CreatureEditorHabitatChanged)
        .padding(10);

    let desc_input = text_input("Description", &editor.description)
        .on_input(Message::CreatureEditorDescriptionChanged)
        .padding(10);

    let danger_input = text_input("Danger (e.g. Low/Medium/High/Extreme)", &editor.danger)
        .on_input(Message::CreatureEditorDangerChanged)
        .padding(10);

    let quick_danger = Row::new()
        .spacing(10)
        .push(ui::outline_button(
            t,
            "Low".to_string(),
            Message::CreatureEditorDangerChanged("Low".to_string()),
        ))
        .push(ui::outline_button(
            t,
            "Medium".to_string(),
            Message::CreatureEditorDangerChanged("Medium".to_string()),
        ))
        .push(ui::outline_button(
            t,
            "High".to_string(),
            Message::CreatureEditorDangerChanged("High".to_string()),
        ))
        .push(ui::outline_button(
            t,
            "Extreme".to_string(),
            Message::CreatureEditorDangerChanged("Extreme".to_string()),
        ));

    let actions = Row::new()
        .spacing(10)
        .push(ui::outline_button(
            t,
            "Cancel".to_string(),
            Message::CreatureEditorCancel,
        ))
        .push(ui::primary_button(
            t,
            "Save".to_string(),
            Message::CreatureEditorSave,
        ));

    let header = Row::new()
        .align_y(Alignment::Center)
        .push(container(text(title).size(16).color(t.foreground)).width(Length::Fill))
        .push(actions);

    let form = Column::new()
        .spacing(10)
        .push(header)
        .push(container(name_input).width(Length::Fill))
        .push(container(kind_input).width(Length::Fill))
        .push(container(habitat_input).width(Length::Fill))
        .push(container(desc_input).width(Length::Fill))
        .push(container(danger_input).width(Length::Fill))
        .push(container(quick_danger).width(Length::Fill))
        .push(
            text("Tip: double-click any creature card below to edit it.")
                .size(10)
                .color(Color::from_rgba8(0xA1, 0xA1, 0xA1, 0.70)),
        );

    ui::card(t, form.into())
}

fn creatures_grid<'a>(t: ui::Tokens, creatures: &'a [Creature]) -> E<'a> {
    let mut col: Column<'a, Message> = Column::new().spacing(14).width(Length::Fill);

    let mut row: Row<'a, Message> = Row::new().spacing(14).width(Length::Fill);
    let mut count: usize = 0;

    for (idx, c) in creatures.iter().enumerate() {
        row = row.push(container(creature_card(t, idx, c)).width(Length::Fill));
        count += 1;

        if count == 3 {
            col = col.push(row);
            row = Row::new().spacing(14).width(Length::Fill);
            count = 0;
        }
    }

    if count != 0 {
        col = col.push(row);
    }

    col.into()
}

fn creature_card<'a>(t: ui::Tokens, index: usize, c: &'a Creature) -> E<'a> {
    let body = Column::new()
        .spacing(6)
        .push(text(&c.name).size(16).color(t.foreground))
        .push(text(&c.kind).size(12).color(t.muted_fg))
        .push(text(format!("Habitat: {}", c.habitat)).size(12).color(t.muted_fg))
        .push(text(&c.description).size(12).color(t.muted_fg))
        .push(text(format!("Danger: {}", c.danger)).size(12).color(t.foreground))
        .push(
            Row::new()
                .spacing(10)
                .push(ui::outline_button(
                    t,
                    "Archive".to_string(),
                    Message::Navigate(crate::app::Route::UniverseList),
                ))
                .push(ui::danger_button(
                    t,
                    "Delete".to_string(),
                    Message::Navigate(crate::app::Route::UniverseList),
                )),
        )
        .push(
            text("Double-click card to edit creature.")
                .size(10)
                .color(Color::from_rgba8(0xA1, 0xA1, 0xA1, 0.55)),
        );

    let card = ui::card(t, body.into());

    // Single click goes to AppState for timing; 2 clicks inside the window becomes a "double-click".
    mouse_area(card)
        .on_press(Message::BestiaryCardClicked(index))
        .into()
}
