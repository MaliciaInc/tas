use iced::{Alignment, Length};
use iced::widget::{container, text, Column, Row};

use crate::app::{AppState, Message, Route, BestiaryMessage, LocationsMessage, UniverseMessage};
use crate::{ui, pages::E};
use crate::state::DemoResetScope;

fn scope_label(scope: DemoResetScope) -> &'static str {
    match scope {
        DemoResetScope::All => "ALL",
        DemoResetScope::Timeline => "Timeline",
        DemoResetScope::Locations => "Locations",
        DemoResetScope::Bestiary => "Bestiary",
        DemoResetScope::PmTools => "PM Tools",
    }
}

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
        .push(ui::outline_button(t, "Go to PM Tools".to_string(), Message::Navigate(Route::PmList)));

    let header = Row::new()
        .align_y(Alignment::Center)
        .push(container(header_left).width(Length::Fill))
        .push(header_right);

    let summary = Column::new()
        .spacing(8)
        .push(text("About this universe").size(12).color(t.muted_fg))
        .push(
            ui::card(
                t,
                Column::new()
                    .spacing(6)
                    .push(text("• Contains custom laws of physics and magic systems.").size(12).color(t.muted_fg))
                    .push(text("• Home to specific species in bestiary, artifacts, etc.").size(12).color(t.muted_fg))
                    .push(text("• Quick links to PM boards, documents and assets related to this universe.").size(12).color(t.muted_fg))
                    .into(),
            )
        );

    let tools = Column::new()
        .spacing(8)
        .push(text("Universe tools").size(12).color(t.muted_fg))
        .push(
            Row::new()
                .spacing(10)
                .push(ui::outline_button(t, "Bestiary".to_string(), Message::Bestiary(BestiaryMessage::Open(universe_id.to_string()))))
                .push(ui::outline_button(t, "Locations".to_string(), Message::Locations(LocationsMessage::Open(universe_id.to_string()))))
                .push(ui::outline_button(t, "Timeline".to_string(), Message::OpenTimeline(universe_id.to_string())))
        );

    let is_arhelis = universe_id == "u-arhelis-01";
    let busy = state.db_inflight.is_some();

    let pending_scope = state
        .pending_demo_reset
        .as_ref()
        .and_then(|(uid, scope)| if uid.as_str() == universe_id { Some(*scope) } else { None });

    let is_pending_reset = pending_scope.is_some();

    let mut body = Column::new()
        .spacing(20)
        .push(header)
        .push(ui::h_divider(t))
        .push(tools)
        .push(summary);

    // Developer Options: ONLY show for Arhelis
    if is_arhelis {
        let inject_row = Row::new()
            .spacing(10)
            .push(if busy {
                // Locked state: show non-clickable label instead of a button
                ui::card(
                    t,
                    text("Inject Demo Data (busy)").size(12).color(t.muted_fg).into(),
                )
            } else {
                ui::primary_button(
                    t,
                    "Inject Demo Data".to_string(),
                    Message::Universe(UniverseMessage::InjectDemoData(universe_id.to_string())),
                )
            });

        let reset_row = if busy {
            // Locked state: show a clear message + non-interactive labels
            Column::new()
                .spacing(6)
                .push(text("Dev Tools are locked while a DB action is running.").size(12).color(t.muted_fg))
                .push(
                    Row::new()
                        .spacing(10)
                        .push(ui::card(t, text("Reset ALL (busy)").size(12).color(t.muted_fg).into()))
                        .push(ui::card(t, text("Reset Timeline (busy)").size(12).color(t.muted_fg).into()))
                        .push(ui::card(t, text("Reset Locations (busy)").size(12).color(t.muted_fg).into()))
                        .push(ui::card(t, text("Reset Bestiary (busy)").size(12).color(t.muted_fg).into()))
                        .push(ui::card(t, text("Reset PM Tools (busy)").size(12).color(t.muted_fg).into()))
                )
        } else {
            Column::new()
                .spacing(6)
                .push(
                    Row::new()
                        .spacing(10)
                        .push(ui::danger_button(
                            t,
                            "Reset ALL".to_string(),
                            Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::All)),
                        ))
                        .push(ui::outline_button(
                            t,
                            "Reset Timeline".to_string(),
                            Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::Timeline)),
                        ))
                        .push(ui::outline_button(
                            t,
                            "Reset Locations".to_string(),
                            Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::Locations)),
                        ))
                        .push(ui::outline_button(
                            t,
                            "Reset Bestiary".to_string(),
                            Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::Bestiary)),
                        ))
                        .push(ui::outline_button(
                            t,
                            "Reset PM Tools".to_string(),
                            Message::Universe(UniverseMessage::ResetDemoPrompt(universe_id.to_string(), DemoResetScope::PmTools)),
                        ))
                )
        };

        let confirm_row = if busy {
            Row::new()
                .spacing(10)
                .push(text("Confirmation disabled (busy).").size(12).color(t.muted_fg))
        } else if is_pending_reset {
            let label = format!(
                "Confirm Reset {}",
                scope_label(pending_scope.unwrap())
            );
            Row::new()
                .spacing(10)
                .push(ui::outline_button(
                    t,
                    "Cancel".to_string(),
                    Message::Universe(UniverseMessage::ResetDemoCancel),
                ))
                .push(ui::primary_button(
                    t,
                    label,
                    Message::Universe(UniverseMessage::ResetDemoConfirm),
                ))
        } else {
            Row::new()
                .spacing(10)
                .push(text("Choose a reset option above.").size(12).color(t.muted_fg))
        };

        let debug_zone = Column::new()
            .spacing(10)
            .push(text("Developer Options (Arhelis only)").size(12).color(t.muted_fg))
            .push(inject_row)
            .push(reset_row)
            .push(confirm_row);

        body = body
            .push(ui::h_divider(t))
            .push(debug_zone);
    }

    body = body.width(Length::Fill);
    ui::page_padding(body.into())
}
