use iced::{border, Alignment, Border, Color, Length};
use iced::widget::{container, text, Column, Row, Space};

use crate::app::{AppState, Message, Route};
use crate::{pages::E, ui};

pub fn overview<'a>(_state: &'a AppState, t: ui::Tokens) -> E<'a> {
    // Header row: page title on the left, one primary action on the right.
    let head = Row::new()
        .align_y(Alignment::Center)
        .push(ui::section_title(t, "Overview".to_string(), None))
        .push(container(Space::new()).width(Length::Fill))
        .push(ui::primary_button(
            t,
            "Open universes".to_string(),
            Message::Navigate(Route::UniverseList),
        ));

    // Welcome card
    let welcome = ui::card(
        t,
        Column::new()
            .spacing(6)
            .push(text("Welcome back.").size(18).color(t.foreground))
            .into(),
    );

    // Grids layout
    let grid_top = Row::new()
        .spacing(14)
        .push(recents_card_universe(t))
        .push(recents_card_forge(t))
        .width(Length::Fill);

    let grid_bottom = Row::new()
        .spacing(14)
        .push(recents_card_pm(t))
        .push(recents_card_assets(t))
        .width(Length::Fill);

    let body = Column::new()
        .spacing(14)
        .push(head)
        .push(welcome)
        .push(grid_top)
        .push(grid_bottom)
        .width(Length::Fill);

    // Envuelve el contenido en el padding estándar de la app
    ui::page_padding(body.into())
}

fn recents_card_universe(t: ui::Tokens) -> iced::Element<'static, Message> {
    recents_card(
        t,
        "Universe — Recents",
        "Latest worldbuilding artifacts.",
        &[
            ("Arhelis — Core Lore", "Updated 2025-09-20", "World"),
            ("Runomicon — Glyphs v2", "Updated 2025-09-18", "Magic"),
            ("Bestiary — Shadows", "Updated 2025-09-15", "Creature"),
            ("Factions — Marekhan", "Updated 2025-09-12", "Faction"),
            ("Locations — Unharier", "Updated 2025-09-10", "Location"),
        ],
    )
}

fn recents_card_forge(t: ui::Tokens) -> iced::Element<'static, Message> {
    recents_card(
        t,
        "The Forge — Recents",
        "Novels and outlines.",
        &[
            ("Novel: Eventum Arhalen", "Updated 2025-09-20", "Novel"),
            ("Novella: The Unseen Tide", "Updated 2025-09-16", "Novella"),
            ("Outline: Book II", "Updated 2025-09-14", "Outline"),
            ("Scene: The Oath", "Updated 2025-09-11", "Scene"),
        ],
    )
}

fn recents_card_pm(t: ui::Tokens) -> iced::Element<'static, Message> {
    recents_card(
        t,
        "PM Tools — Recents",
        "Boards, columns and cards.",
        &[
            ("TAS — PM Roadmap", "Updated 2025-09-21", "Roadmap"),
            ("Universe Cleanup Sprint", "Updated 2025-09-17", "Sprint"),
            ("Assets Backlog", "Updated 2025-09-13", "Backlog"),
            ("Forge Editing Tasks", "Updated 2025-09-11", "Tasks"),
        ],
    )
}

fn recents_card_assets(t: ui::Tokens) -> iced::Element<'static, Message> {
    recents_card(
        t,
        "Assets — Recents",
        "Uploaded files and references.",
        &[
            ("Map — Central Plateau.png", "Updated 2025-09-22", "Image"),
            ("Glyph Sheet v3.svg", "Updated 2025-09-20", "Vector"),
            ("Theme Moodboard.pdf", "Updated 2025-09-15", "Doc"),
            ("Faction Seals.zip", "Updated 2025-09-09", "Archive"),
        ],
    )
}

fn recents_card<'a>(
    t: ui::Tokens,
    title: &'a str,
    subtitle: &'a str,
    items: &'a [(&'a str, &'a str, &'a str)],
) -> E<'a> {
    let head = Row::new()
        .align_y(Alignment::Center)
        .push(text(title).size(14).color(t.foreground))
        .push(container(Space::new()).width(Length::Fill))
        .push(ui::ghost_button(
            t,
            "View all →".to_string(),
            Message::Navigate(Route::Overview),
        ));

    let mut list = Column::new().spacing(10);

    for (name, updated, tag) in items {
        let left = Column::new()
            .spacing(2)
            .push(text(*name).size(12).color(t.foreground))
            .push(text(*updated).size(10).color(Color::from_rgba8(0xA1, 0xA1, 0xA1, 0.70)));

        let tag_pill = container(text(*tag).size(10).color(t.foreground))
            .padding([4, 8])
            .style(move |_| {
                let mut s = ui::container_style(t.active_bg, t.foreground);
                s.border = Border {
                    color: Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.10),
                    width: 1.0,
                    radius: border::Radius::from(999.0),
                };
                s
            });

        let row_item = Row::new()
            .align_y(Alignment::Center)
            .push(container(left).width(Length::Fill))
            .push(tag_pill);

        list = list.push(row_item);
    }

    let body = Column::new()
        .spacing(8)
        .push(head)
        .push(text(subtitle).size(11).color(Color::from_rgba8(0xA1, 0xA1, 0xA1, 0.70)))
        .push(list);

    ui::card(t, body.into())
}
