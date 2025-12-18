// src/pages/overview.rs
// Overview dashboard (no "creative cockpit", no "Build worlds..." line).

use iced::{Alignment, Element, Length};
use iced::widget::{Column, Row, Space, text};

use crate::app::{AppState, Message, Route};
use crate::ui;

pub fn overview(state: &AppState, t: ui::Tokens) -> Element<'static, Message> {
    let header_row = Row::new()
        .align_y(Alignment::Center)
        .push(ui::section_title(t, "Overview".to_string(), None))
        .push(Space::with_width(Length::Fill))
        .push(ui::primary_button(
            t,
            "Open universes".to_string(),
            Message::Navigate(Route::UniverseList),
        ));

    let stats = Row::new()
        .spacing(10)
        .push(ui::stat_pill(t, state.universes.len() as u32, "Universes"))
        .push(ui::stat_pill(t, state.creatures.len() as u32, "Creatures"))
        .push(ui::stat_pill(t, 0, "Drafts"))
        .push(ui::stat_pill(t, 0, "Tasks"));

    let quick = Row::new()
        .spacing(10)
        .push(ui::chip(t, "The Forge".to_string()))
        .push(ui::chip(t, "PM Tools".to_string()))
        .push(ui::chip(t, "Assets".to_string()));

    let welcome = ui::card(
        t,
        Column::new()
            .spacing(12)
            .push(text("Welcome back.").size(20))
            .push(ui::h_divider(t))
            .push(stats)
            .push(quick)
            .into(),
    );

    // Keep your existing layout structure; this is a clean, premium baseline.
    let layout = Column::new()
        .spacing(18)
        .push(header_row)
        .push(welcome);

    layout.into()
}
