use iced::{Color, Element, Length, Theme, Vector};
use iced::widget::{container, scrollable, text, Column, Row, Stack};

use crate::{pages, ui};
use super::{AppState, Message, Route, PmState};

// --- DEBUG OVERLAY (A) ---
fn debug_overlay(state: &AppState, t: ui::Tokens) -> Element<'_, Message> {
    let inflight = match &state.db_inflight {
        None => "None".to_string(),
        Some(a) => format!("{:?}", a),
    };

    let schema = match state.debug_schema_version {
        None => "…".to_string(),
        Some(v) => v.to_string(),
    };

    let route = format!("{:?}", state.route);

    let counts = format!(
        "Universes={} Creatures={} Locations={} Eras={} Events={} Snapshots={} Issues={}",
        state.universes.len(),
        state.creatures.len(),
        state.locations.len(),
        state.timeline_eras.len(),
        state.timeline_events.len(),
        state.snapshots.len(),
        state.integrity_issues.len(),
    );

    let pending_reset = match &state.pending_demo_reset {
        None => "None".to_string(),
        Some((uid, scope)) => format!("{} / {:?}", uid, scope),
    };

    let mut issues_col = Column::new().spacing(4);
    if state.integrity_issues.is_empty() {
        issues_col = issues_col.push(
            text("No integrity issues detected.")
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        );
    } else {
        for (i, issue) in state.integrity_issues.iter().take(8).enumerate() {
            issues_col = issues_col.push(
                text(format!("{}. {}", i + 1, issue))
                    .size(12)
                    .style(move |_| iced::widget::text::Style { color: Some(t.foreground) }),
            );
        }
        if state.integrity_issues.len() > 8 {
            issues_col = issues_col.push(
                text(format!("…and {} more", state.integrity_issues.len() - 8))
                    .size(12)
                    .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
            );
        }
    }

    let content = Column::new()
        .spacing(10)
        .push(
            Row::new()
                .spacing(12)
                .push(
                    text("Debug Overlay")
                        .size(16)
                        .style(move |_| iced::widget::text::Style { color: Some(t.foreground) }),
                )
                .push(
                    text("(Toggle from Universe Detail)")
                        .size(12)
                        .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
                ),
        )
        .push(
            text(format!("schema_version={}", schema))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(format!("db_inflight={}", inflight))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(format!("pending_reset={}", pending_reset))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(format!("route={}", route))
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(
            text(counts)
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(ui::h_divider(t))
        .push(
            text("Integrity issues (top 8):")
                .size(12)
                .style(move |_| iced::widget::text::Style { color: Some(t.muted_fg) }),
        )
        .push(issues_col);

    let panel = container(content)
        .padding(14)
        .width(Length::Fixed(820.0))
        .style(move |_: &Theme| {
            let mut s = ui::container_style(ui::alpha(t.shell_b, 0.98), t.foreground);
            s.border.color = t.accent;
            s.border.width = 2.0;
            s.border.radius = 12.0.into();
            s.shadow = iced::Shadow {
                color: Color::BLACK,
                offset: Vector::new(0.0, 12.0),
                blur_radius: 24.0,
            };
            s
        });

    container(panel)
        .padding(iced::Padding {
            top: 20.0,
            left: 260.0,
            right: 20.0,
            bottom: 20.0,
        })
        .into()
}

// --- VIEW DISPATCHER ---
pub fn view(state: &AppState) -> Element<'_, Message> {
    let t = ui::Tokens::nub_dark();

    // 1) LAUNCHER MODE
    if state.active_project.is_none() {
        return pages::launcher::launcher_view(state, t);
    }

    // 2) STUDIO MODE
    let sidebar = ui::sidebar(state, t);
    let header = ui::header(state, t);

    let page: Element<'_, Message> = match &state.route {
        Route::Overview => pages::overview(state, t),
        Route::Workspaces => pages::workspaces::workspaces_page(state, t),
        Route::UniverseList => pages::universe_list(state, t),
        Route::UniverseDetail { universe_id } => pages::universe_detail(state, t, universe_id),
        Route::Bestiary { universe_id } => pages::bestiary(state, t, universe_id),
        Route::Locations { universe_id } => pages::locations::locations(state, t, universe_id),
        Route::Timeline { universe_id } => pages::timeline::timeline(state, t, universe_id),
        Route::PmList => pages::pm_list::pm_list(state, t),
        Route::PmBoard { .. } => pages::pm_board::pm_board(state, t, &state.pm_data),

        // Forge real view
        Route::Forge => pages::the_forge_view(state, t),

        Route::Assets => pages::assets_stub(state, t),
        Route::Account => pages::account_stub(state, t),
    };

    let right = Column::new()
        .spacing(14)
        .push(header)
        .push(scrollable(page).width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill);

    let root = Row::new()
        .spacing(0)
        .push(container(sidebar).width(Length::Fixed(240.0)).height(Length::Fill))
        .push(right)
        .width(Length::Fill)
        .height(Length::Fill);

    let mut stack = Stack::new().push(ui::shell(t, root.into()));

    // OVERLAYS (Modals)
    if let PmState::Editing {
        title,
        description,
        priority,
        card_id,
        ..
    } = &state.pm_state
    {
        let is_new = card_id.is_none();
        stack = stack.push(crate::pages::pm_board::render_modal(
            t,
            title,
            description,
            priority,
            is_new,
        ));
    }
    if let Some(editor) = &state.creature_editor {
        stack = stack.push(pages::bestiary::render_creature_modal(t, editor, &state.locations));
    }
    if let Some(editor) = &state.location_editor {
        stack = stack.push(pages::locations::render_location_modal(t, editor));
    }
    if let Some(editor) = &state.event_editor {
        stack = stack.push(pages::timeline::render_event_modal(t, editor, &state.locations));
    }
    if let Some(editor) = &state.era_editor {
        stack = stack.push(pages::timeline::render_era_modal(t, editor));
    }

    // Dragging Ghost
    if let PmState::Dragging {
        card,
        current_cursor,
        active,
        ..
    } = &state.pm_state
    {
        if *active {
            let ghost = container(
                text(&card.title)
                    .size(14)
                    .style(move |_| iced::widget::text::Style { color: Some(t.foreground) }),
            )
                .padding(12)
                .width(Length::Fixed(280.0))
                .style(move |_: &Theme| {
                    let mut s = ui::container_style(t.shell_b, t.foreground);
                    s.border.color = t.accent;
                    s.border.width = 2.0;
                    s.border.radius = 6.0.into();
                    s.background = Some(ui::alpha(t.background, 0.9).into());
                    s.shadow = iced::Shadow {
                        color: Color::BLACK,
                        offset: Vector::new(0.0, 10.0),
                        blur_radius: 20.0,
                    };
                    s
                });

            stack = stack.push(container(ghost).padding(iced::Padding {
                top: current_cursor.y + 10.0,
                left: current_cursor.x + 10.0,
                bottom: 0.0,
                right: 0.0,
            }));
        }
    }

    // Debug Overlay (above modals)
    if state.debug_overlay_open {
        stack = stack.push(debug_overlay(state, t));
    }

    // Toasts
    if !state.toasts.is_empty() {
        stack = stack.push(ui::toasts_overlay(t, &state.toasts));
    }

    stack.into()
}
