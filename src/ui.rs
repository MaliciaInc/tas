// src/ui.rs
// UI helpers + styling tokens for TAS.
// (Code/comments in English as requested.)

use iced::{
    border, gradient,
    widget::{button, container, scrollable, text, Column, Row, Space},
    Alignment, Background, Color, Element, Length, Shadow, Theme, Vector,
};

use gradient::{Gradient, Linear};

use crate::app::{AppState, Message, Route};

type E<'a> = Element<'a, Message>;

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub background: Color,
    pub shell_a: Color,
    pub shell_b: Color,

    pub card: Color,
    pub popover: Color,
    pub sidebar_bg: Color,

    pub border: Color,
    pub foreground: Color,
    pub foreground_muted: Color,

    pub accent_purple: Color, // kept for future use (not used for buttons/active now)
    pub accent_teal: Color,   // kept for future use (not used for buttons/active now)
    pub danger: Color,

    pub shadow: Color,
}

pub fn alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

impl Tokens {
    pub fn nub_dark() -> Self {
        // Slightly brighter right-side feel comes mostly from shell gradient + higher opacities.
        // Sidebar now uses the same tonal family as cards (no transparent strip).
        Self {
            background: Color::from_rgba8(0x00, 0x00, 0x00, 1.0),

            // +~5% brightness vs previous to avoid "too black" while staying premium.
            shell_a: Color::from_rgba8(0x2E, 0x2D, 0x2D, 1.0),
            shell_b: Color::from_rgba8(0x23, 0x22, 0x22, 1.0),

            card: Color::from_rgba8(0x1D, 0x1D, 0x1D, 0.86),
            popover: Color::from_rgba8(0x1D, 0x1D, 0x1D, 0.92),

            // IMPORTANT: sidebar now solid (same family as cards) to remove the light strip.
            sidebar_bg: Color::from_rgba8(0x1D, 0x1D, 0x1D, 0.90),

            border: alpha(Color::WHITE, 0.10),

            foreground: Color::from_rgb8(0xEF, 0xEF, 0xEF),
            foreground_muted: alpha(Color::WHITE, 0.55),

            accent_purple: Color::from_rgb8(0x7C, 0x3A, 0xF2),
            accent_teal: Color::from_rgb8(0x2D, 0xC9, 0xC3),

            danger: Color::from_rgb8(0xE3, 0x5B, 0x5B),

            shadow: alpha(Color::BLACK, 0.40),
        }
    }
}

pub fn container_style(bg: Color, fg: Color) -> container::Style {
    container::Style {
        text_color: Some(fg),
        background: Some(bg.into()),
        border: border::rounded(0.0),
        shadow: Shadow::default(),
    }
}

fn shell_style(t: Tokens) -> container::Style {
    let bg = Background::Gradient(Gradient::Linear(
        Linear::new(0.0).add_stop(0.0, t.shell_a).add_stop(1.0, t.shell_b),
    ));

    container::Style {
        text_color: Some(t.foreground),
        background: Some(bg),
        border: border::rounded(0.0),
        shadow: Shadow::default(),
    }
}

pub fn shell<'a>(t: Tokens, content: E<'a>) -> E<'a> {
    // Full-bleed shell (no "window inside window", no colored circles).
    container(content)
        .padding(0)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| shell_style(t))
        .into()
}

fn card_style(t: Tokens) -> container::Style {
    container::Style {
        text_color: Some(t.foreground),
        background: Some(t.card.into()),
        border: border::Border {
            color: alpha(Color::WHITE, 0.10),
            width: 1.0,
            radius: 18.0.into(),
        },
        shadow: Shadow {
            color: alpha(Color::BLACK, 0.35),
            offset: Vector::new(0.0, 10.0),
            blur_radius: 28.0,
        },
    }
}

pub fn card<'a>(t: Tokens, content: E<'a>) -> E<'a> {
    container(content)
        .padding(18)
        .width(Length::Fill)
        .style(move |_| card_style(t))
        .into()
}

// This is the "Arhelis pill" look — we reuse it for hover/active everywhere.
fn pill_bg_style(t: Tokens) -> container::Style {
    container::Style {
        text_color: Some(t.foreground),
        background: Some(alpha(Color::WHITE, 0.06).into()),
        border: border::Border {
            color: alpha(Color::WHITE, 0.10),
            width: 1.0,
            radius: 999.0.into(),
        },
        shadow: Shadow::default(),
    }
}

pub fn pill<'a>(t: Tokens, content: E<'a>) -> E<'a> {
    container(content)
        .padding([8, 12])
        .style(move |_| pill_bg_style(t))
        .into()
}

pub fn h_divider(t: Tokens) -> E<'static> {
    container(Space::new(Length::Fill, Length::Fixed(1.0)))
        .style(move |_| container_style(alpha(Color::WHITE, 0.06), t.foreground))
        .into()
}

pub fn v_divider(t: Tokens) -> E<'static> {
    container(Space::new(Length::Fixed(1.0), Length::Fill))
        .style(move |_| container_style(alpha(Color::WHITE, 0.06), t.foreground))
        .into()
}

pub fn section_title(t: Tokens, title: String, subtitle: Option<String>) -> E<'static> {
    let mut col = Column::new().spacing(6);
    col = col.push(text(title).size(28));

    if let Some(sub) = subtitle {
        col = col.push(text(sub).size(14).style(move |_| iced::widget::text::Style {
            color: Some(t.foreground_muted),
        }));
    }

    col.into()
}

pub fn primary_button(t: Tokens, label: String, on_press: Message) -> E<'static> {
    // No purple/blue. Same family as the workspace pill, slightly stronger on hover.
    button(text(label).size(14))
        .padding([10, 14])
        .style(move |_: &Theme, status| {
            let (bg_a, bd_a) = match status {
                iced::widget::button::Status::Hovered => (0.08, 0.12),
                iced::widget::button::Status::Pressed => (0.10, 0.14),
                _ => (0.06, 0.10),
            };

            iced::widget::button::Style {
                text_color: t.foreground,
                background: Some(alpha(Color::WHITE, bg_a).into()),
                border: border::Border {
                    color: alpha(Color::WHITE, bd_a),
                    width: 1.0,
                    radius: 999.0.into(),
                },
                shadow: Shadow::default(),
            }
        })
        .on_press(on_press)
        .into()
}

pub fn ghost_button(t: Tokens, label: String, on_press: Message) -> E<'static> {
    button(text(label).size(13))
        .padding([8, 12])
        .style(move |_: &Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => alpha(Color::WHITE, 0.05),
                iced::widget::button::Status::Pressed => alpha(Color::WHITE, 0.07),
                _ => alpha(Color::WHITE, 0.00),
            };

            iced::widget::button::Style {
                text_color: t.foreground_muted,
                background: Some(bg.into()),
                border: border::Border {
                    color: alpha(Color::WHITE, 0.10),
                    width: 1.0,
                    radius: 999.0.into(),
                },
                shadow: Shadow::default(),
            }
        })
        .on_press(on_press)
        .into()
}

pub fn danger_button(t: Tokens, label: String, on_press: Message) -> E<'static> {
    button(text(label).size(13))
        .padding([8, 12])
        .style(move |_: &Theme, status| {
            let bg = match status {
                iced::widget::button::Status::Hovered => alpha(t.danger, 0.18),
                iced::widget::button::Status::Pressed => alpha(t.danger, 0.26),
                _ => alpha(t.danger, 0.12),
            };

            iced::widget::button::Style {
                text_color: t.foreground,
                background: Some(bg.into()),
                border: border::Border {
                    color: alpha(t.danger, 0.55),
                    width: 1.0,
                    radius: 999.0.into(),
                },
                shadow: Shadow::default(),
            }
        })
        .on_press(on_press)
        .into()
}

pub fn small_tag(t: Tokens, label: &'static str) -> E<'static> {
    container(text(label).size(12))
        .padding([6, 10])
        .style(move |_| container::Style {
            text_color: Some(t.foreground_muted),
            background: Some(alpha(Color::WHITE, 0.05).into()),
            border: border::Border {
                color: alpha(Color::WHITE, 0.08),
                width: 1.0,
                radius: 999.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

pub fn chip(t: Tokens, label: String) -> E<'static> {
    container(text(label).size(12))
        .padding([6, 10])
        .style(move |_| container::Style {
            text_color: Some(t.foreground_muted),
            background: Some(alpha(Color::WHITE, 0.04).into()),
            border: border::Border {
                color: alpha(Color::WHITE, 0.08),
                width: 1.0,
                radius: 999.0.into(),
            },
            shadow: Shadow::default(),
        })
        .into()
}

pub fn stat_pill(t: Tokens, value: u32, label: &str) -> E<'static> {
    let left = container(text(value.to_string()).size(12))
        .padding([4, 10])
        .style(move |_| container::Style {
            text_color: Some(t.foreground),
            background: Some(alpha(Color::WHITE, 0.05).into()),
            border: border::Border {
                color: alpha(Color::WHITE, 0.08),
                width: 1.0,
                radius: 999.0.into(),
            },
            shadow: Shadow::default(),
        });

    let right = container(text(label.to_string()).size(12))
        .padding([4, 10])
        .style(move |_| container::Style {
            text_color: Some(t.foreground_muted),
            background: Some(alpha(Color::WHITE, 0.02).into()),
            border: border::Border {
                color: alpha(Color::WHITE, 0.06),
                width: 1.0,
                radius: 999.0.into(),
            },
            shadow: Shadow::default(),
        });

    Row::new().spacing(8).push(left).push(right).into()
}

pub fn empty_state(t: Tokens, title: &str, body: &str) -> E<'static> {
    card(
        t,
        Column::new()
            .spacing(8)
            .push(text(title.to_string()).size(16))
            .push(text(body.to_string()).size(13).style(move |_| iced::widget::text::Style {
                color: Some(t.foreground_muted),
            }))
            .into(),
    )
}

// ----------------------------
// Lucide icons (SVG, offline)
// ----------------------------

#[derive(Clone, Copy)]
enum LucideIcon {
    Menu,
    Dashboard,
    Folder,
    Globe,
    Hammer,
    Kanban,
    Package,
    Settings,
}

const SVG_MENU: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="4" y1="6" x2="20" y2="6"/><line x1="4" y1="12" x2="20" y2="12"/><line x1="4" y1="18" x2="20" y2="18"/></svg>"#;

const SVG_DASHBOARD: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="9" rx="1"/><rect x="14" y="3" width="7" height="5" rx="1"/><rect x="14" y="10" width="7" height="11" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/></svg>"#;

const SVG_FOLDER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 4H4a2 2 0 0 0-2 2v3h20V8a2 2 0 0 0-2-2h-8z"/><path d="M2 9v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9z"/></svg>"#;

const SVG_GLOBE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15 15 0 0 1 0 20"/><path d="M12 2a15 15 0 0 0 0 20"/></svg>"#;

const SVG_HAMMER: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 7l3 3"/><path d="M5 21l10-10"/><path d="M7 3l4 4"/><path d="M6 4l2-2 5 5-2 2z"/><path d="M13 7l6 6-2 2-6-6z"/></svg>"#;

const SVG_KANBAN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M7 7v9"/><path d="M12 7v5"/><path d="M17 7v7"/></svg>"#;

const SVG_PACKAGE: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16.5 9.4 7.5 4.2"/><path d="M21 16V8a2 2 0 0 0-1-1.7l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.7l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="M3.3 7.3 12 12l8.7-4.7"/><path d="M12 22V12"/></svg>"#;

const SVG_SETTINGS: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 15.5a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z"/><path d="M19.4 15a7.9 7.9 0 0 0 .1-1 7.9 7.9 0 0 0-.1-1l2-1.6-2-3.4-2.4 1a8.2 8.2 0 0 0-1.7-1L15 3h-6l-.3 2.4a8.2 8.2 0 0 0-1.7 1l-2.4-1-2 3.4 2 1.6a7.9 7.9 0 0 0-.1 1 7.9 7.9 0 0 0 .1 1l-2 1.6 2 3.4 2.4-1a8.2 8.2 0 0 0 1.7 1L9 21h6l.3-2.4a8.2 8.2 0 0 0 1.7-1l2.4 1 2-3.4Z"/></svg>"#;

fn lucide_handle(icon: LucideIcon) -> iced::widget::svg::Handle {
    let svg = match icon {
        LucideIcon::Menu => SVG_MENU,
        LucideIcon::Dashboard => SVG_DASHBOARD,
        LucideIcon::Folder => SVG_FOLDER,
        LucideIcon::Globe => SVG_GLOBE,
        LucideIcon::Hammer => SVG_HAMMER,
        LucideIcon::Kanban => SVG_KANBAN,
        LucideIcon::Package => SVG_PACKAGE,
        LucideIcon::Settings => SVG_SETTINGS,
    };

    iced::widget::svg::Handle::from_memory(svg.as_bytes())
}

fn lucide_icon(t: Tokens, icon: LucideIcon, size: f32) -> E<'static> {
    let handle = lucide_handle(icon);

    iced::widget::svg(handle)
        .width(Length::Fixed(size))
        .height(Length::Fixed(size))
        .style(move |_: &Theme, _status: iced::widget::svg::Status| iced::widget::svg::Style {
            color: Some(t.foreground),
        })
        .into()
}

fn icon_button(t: Tokens, icon: LucideIcon, active: bool, route: Route, on_press: Message) -> E<'static> {
    let content = container(lucide_icon(t, icon, 18.0))
        .width(Length::Fixed(44.0))
        .height(Length::Fixed(44.0))
        .align_x(Alignment::Center)
        .align_y(Alignment::Center)
        .style(move |_| {
            if active {
                // EXACT same look as workspace pill.
                pill_bg_style(t)
            } else {
                container::Style {
                    text_color: Some(t.foreground),
                    background: Some(alpha(Color::WHITE, 0.00).into()),
                    border: border::rounded(14.0),
                    shadow: Shadow::default(),
                }
            }
        });

    button(content)
        .padding(0)
        .width(Length::Fixed(44.0))
        .height(Length::Fixed(44.0))
        .style(move |_: &Theme, status| {
            let is_hover = matches!(status, iced::widget::button::Status::Hovered);
            let is_pressed = matches!(status, iced::widget::button::Status::Pressed);

            let bg = if active {
                // keep active stable; slight boost on hover/press
                if is_pressed {
                    alpha(Color::WHITE, 0.10)
                } else if is_hover {
                    alpha(Color::WHITE, 0.08)
                } else {
                    alpha(Color::WHITE, 0.06)
                }
            } else if is_pressed {
                alpha(Color::WHITE, 0.06)
            } else if is_hover {
                alpha(Color::WHITE, 0.05)
            } else {
                alpha(Color::WHITE, 0.00)
            };

            let bd = if active || is_hover || is_pressed {
                alpha(Color::WHITE, 0.10)
            } else {
                alpha(Color::WHITE, 0.00)
            };

            iced::widget::button::Style {
                text_color: t.foreground,
                background: Some(bg.into()),
                border: border::Border {
                    color: bd,
                    width: 1.0,
                    radius: 14.0.into(),
                },
                shadow: Shadow::default(),
            }
        })
        .on_press(on_press)
        .into()
}

pub fn sidebar(state: &AppState, t: Tokens) -> E<'static> {
    let top_menu = icon_button(
        t,
        LucideIcon::Menu,
        true, // always visible as a pill button (like your preference)
        Route::Overview,
        Message::Navigate(Route::Overview),
    );

    let nav = Column::new()
        .spacing(10)
        .push(icon_button(
            t,
            LucideIcon::Dashboard,
            matches!(state.route, Route::Overview),
            Route::Overview,
            Message::Navigate(Route::Overview),
        ))
        .push(icon_button(
            t,
            LucideIcon::Folder,
            matches!(state.route, Route::Workspaces),
            Route::Workspaces,
            Message::Navigate(Route::Workspaces),
        ))
        .push(icon_button(
            t,
            LucideIcon::Globe,
            matches!(
                state.route,
                Route::UniverseList
                    | Route::UniverseDetail { .. }
                    | Route::Bestiary { .. }
                    | Route::Timeline { .. }
            ),
            Route::UniverseList,
            Message::Navigate(Route::UniverseList),
        ))
        .push(icon_button(
            t,
            LucideIcon::Hammer,
            matches!(state.route, Route::Forge),
            Route::Forge,
            Message::Navigate(Route::Forge),
        ))
        .push(icon_button(
            t,
            LucideIcon::Kanban,
            matches!(state.route, Route::PmTools),
            Route::PmTools,
            Message::Navigate(Route::PmTools),
        ))
        .push(icon_button(
            t,
            LucideIcon::Package,
            matches!(state.route, Route::Assets),
            Route::Assets,
            Message::Navigate(Route::Assets),
        ));

    let bottom = Column::new()
        .spacing(10)
        .push(Space::with_height(Length::Fill))
        // Account becomes gear/settings
        .push(icon_button(
            t,
            LucideIcon::Settings,
            matches!(state.route, Route::Account),
            Route::Account,
            Message::Navigate(Route::Account),
        ));

    let layout = Column::new()
        .spacing(10)
        .push(top_menu)
        .push(Space::with_height(Length::Fixed(10.0)))
        .push(nav)
        .push(bottom)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Alignment::Center);

    container(layout)
        .padding([16, 12])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| container_style(t.sidebar_bg, t.foreground))
        .into()
}

pub fn header(state: &AppState, t: Tokens) -> E<'static> {
    let brand = Column::new()
        .spacing(4)
        .push(text("Titan Architect Studio").size(16))
        .push(
            text("Where Reality Begins.")
                .size(12)
                .style(move |_| iced::widget::text::Style {
                    color: Some(t.foreground_muted),
                }),
        );

    let workspace = pill(
        t,
        text(state.active_workspace.clone())
            .size(13)
            .into(),
    );

    let row = Row::new()
        .align_y(Alignment::Center)
        .spacing(14)
        .push(brand)
        .push(Space::with_width(Length::Fill))
        .push(workspace);

    container(row)
        .padding([14, 16])
        .width(Length::Fill)
        .style(move |_| container_style(alpha(Color::WHITE, 0.02), t.foreground))
        .into()
}
