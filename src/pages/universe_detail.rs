use iced::{Alignment, Length};
use iced::widget::{container, text, Column, Row};

use crate::app::{AppState, Message, Route};
use crate::{ui, pages::E};

pub fn universe_detail<'a>(state: &'a AppState, t: ui::Tokens, universe_id: &'a str) -> E<'a> {
    let u = state.universes.iter().find(|x| x.id == universe_id);

    let (name, desc) = match u {
        Some(u) => (u.name.clone(), u.description.clone()),
        None => ("Unknown".to_string(), "".to_string()),
    };

    let header_left = Column::new()
        .spacing(4)
        .push(text(name.clone()).size(26).color(t.foreground))
        .push(text(desc).size(12).color(t.muted_fg))
        .push(text("Status: Active").size(12).color(t.muted_fg));

    let header_right = Row::new()
        .spacing(10)
        .push(ui::outline_button(t, "Back to universes".to_string(), Message::BackToUniverses))
        .push(ui::outline_button(t, "Go to PM Tools".to_string(), Message::Navigate(Route::PmTools)));

    let header = Row::new()
        .align_y(Alignment::Center)
        .push(container(header_left).width(Length::Fill))
        .push(header_right);

    let info = ui::card(
        t,
        Column::new()
            .spacing(6)
            .push(text("This is the starting point for your universe workspace.").size(12).color(t.muted_fg))
            .push(text("In future iterations, this page will host:").size(12).color(t.muted_fg))
            .push(text("• Key lore overview (high-level notes, pitch, themes)").size(12).color(t.muted_fg))
            .push(text("• Entry points into timelines, maps, bestiary, artifacts, etc.").size(12).color(t.muted_fg))
            .push(text("• Quick links to PM boards, documents and assets related to this universe.").size(12).color(t.muted_fg))
            .into(),
    );

    let tools = Column::new()
        .spacing(8)
        .push(text("Universe tools").size(12).color(t.muted_fg))
        .push(
            Row::new()
                .spacing(10)
                .push(ui::outline_button(t, "Bestiary".to_string(), Message::OpenBestiary(universe_id.to_string())))
                .push(ui::outline_button(t, "Timeline".to_string(), Message::OpenTimeline(universe_id.to_string()))),
        );

    let linked_pm = Column::new()
        .spacing(8)
        .push(text("Linked PM boards").size(12).color(t.muted_fg))
        .push(
            ui::card(
                t,
                container(text("No PM boards linked to this universe yet.").size(12).color(t.muted_fg))
                    .width(Length::Fill)
                    .padding([14, 14])
                    .into(),
            ),
        );

    let body = Column::new()
        .spacing(14)
        .push(header)
        .push(info)
        .push(tools)
        .push(linked_pm)
        .width(Length::Fill);

    ui::page_padding(body.into())
}
