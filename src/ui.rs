use iced::{
    border, gradient, Alignment, Background, Border, Color, Element, Length, Padding, Radians,
    Shadow, Theme, Vector,
};
use iced::widget::{button, container, text, Column, Row, Space};

use crate::app::{AppState, Message, Route, APP_NAME};

// Simplificamos el tipo para uso interno
pub type E<'a> = Element<'a, Message>;

const APP_SLOGAN: &str = "Where Reality Begins.";

#[derive(Debug, Clone, Copy)]
pub struct Tokens {
    pub background: Color,
    pub foreground: Color,
    pub muted_fg: Color,

    // Surfaces
    pub shell_a: Color,
    pub shell_b: Color,
    pub card: Color,
    pub popover: Color,
    pub sidebar_bg: Color,

    // Lines & states
    pub border: Color,
    pub input_border: Color,
    pub hover_bg: Color,
    pub active_bg: Color,

    // Radii
    pub radius_xl: f32,
    pub radius_lg: f32,
}

impl Tokens {
    pub fn nub_dark() -> Self {
        let white = Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0);

        Self {
            background: Color::from_rgba8(0x00, 0x00, 0x00, 1.0),
            foreground: Color::from_rgba8(0xF2, 0xF2, 0xF2, 1.0),
            muted_fg: Color::from_rgba8(0xA7, 0xA8, 0xAB, 0.82),

            // Shell Gradient
            shell_a: Color::from_rgba8(0x27, 0x26, 0x26, 1.0),
            shell_b: Color::from_rgba8(0x1E, 0x1D, 0x1D, 1.0),

            // Surfaces
            card: Color::from_rgba8(0x1D, 0x1D, 0x1D, 0.86),
            popover: Color::from_rgba8(0x1D, 0x1D, 0x1D, 0.94),

            // Sidebar: Totalmente opaco para evitar mezcla de colores
            sidebar_bg: Color::from_rgba8(0x1B, 0x1B, 0x1B, 1.0),

            border: alpha(white, 0.08),
            input_border: alpha(white, 0.14),

            hover_bg: alpha(white, 0.05),
            active_bg: alpha(white, 0.07),

            radius_xl: 20.0,
            radius_lg: 14.0,
        }
    }
}

fn alpha(mut c: Color, a: f32) -> Color {
    c.a = a;
    c
}

// Estilo base para contenedores
pub fn container_style(bg: Color, fg: Color) -> iced::widget::container::Style {
    iced::widget::container::Style {
        text_color: Some(fg),
        background: Some(Background::Color(bg)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::Radius::from(0.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

fn shell_style(t: Tokens) -> iced::widget::container::Style {
    let grad = gradient::Linear::new(Radians::PI / 4.0)
        .add_stop(0.0, t.shell_a)
        .add_stop(1.0, t.shell_b);

    iced::widget::container::Style {
        text_color: Some(t.foreground),
        background: Some(grad.into()),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: border::Radius::from(0.0),
        },
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Fondo principal de la ventana
pub fn shell<'a>(t: Tokens, content: E<'a>) -> E<'a> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| shell_style(t))
        .into()
}

/// Padding estándar para las páginas (Overview, Forge, etc.)
pub fn page_padding<'a>(content: E<'a>) -> E<'a> {
    container(content)
        .padding(Padding {
            top: 18.0,
            right: 24.0,
            bottom: 22.0,
            left: 18.0,
        })
        .width(Length::Fill)
        .into()
}

pub fn section_title(t: Tokens, title: String, subtitle: Option<String>) -> Element<'static, Message> {
    let mut col = Column::new().spacing(4);
    col = col.push(text(title).size(28).color(t.foreground));
    if let Some(s) = subtitle {
        col = col.push(text(s).size(12).color(t.muted_fg));
    }
    col.into()
}

pub fn card<'a>(t: Tokens, content: E<'a>) -> E<'a> {
    container(content)
        .padding(16)
        .width(Length::Fill)
        .style(move |_| {
            let mut s = container_style(t.card, t.foreground);
            // Borde sutil alrededor de las cards
            s.border = Border {
                color: t.border,
                width: 1.0,
                radius: border::Radius::from(t.radius_xl),
            };
            // Sombra para dar profundidad
            s.shadow = Shadow {
                color: Color::from_rgba8(0x00, 0x00, 0x00, 0.38),
                offset: Vector::new(0.0, 18.0),
                blur_radius: 36.0,
            };
            s
        })
        .into()
}

// Botones

pub fn outline_button(t: Tokens, label: String, on_press: Message) -> Element<'static, Message> {
    button(text(label).size(12).color(t.foreground))
        .padding([8, 12])
        .style(move |_: &Theme, status| {
            let mut s = iced::widget::button::Style::default();
            let bg = match status {
                iced::widget::button::Status::Hovered => t.hover_bg,
                iced::widget::button::Status::Pressed => t.active_bg,
                _ => Color::TRANSPARENT,
            };
            s.background = Some(Background::Color(bg));
            s.border = Border {
                color: t.border,
                width: 1.0,
                radius: border::Radius::from(999.0),
            };
            s.text_color = t.foreground;
            s
        })
        .on_press(on_press)
        .into()
}

pub fn ghost_button(t: Tokens, label: String, on_press: Message) -> Element<'static, Message> {
    button(text(label).size(12).color(t.muted_fg))
        .padding([6, 10])
        .style(move |_: &Theme, status| {
            let mut s = iced::widget::button::Style::default();
            let bg = match status {
                iced::widget::button::Status::Hovered => t.hover_bg,
                iced::widget::button::Status::Pressed => t.active_bg,
                _ => Color::TRANSPARENT,
            };
            s.background = Some(Background::Color(bg));
            s.text_color = t.muted_fg;
            s
        })
        .on_press(on_press)
        .into()
}

pub fn primary_button(t: Tokens, label: String, on_press: Message) -> Element<'static, Message> {
    button(text(label).size(12).color(t.foreground))
        .padding([8, 12])
        .style(move |_: &Theme, status| {
            let mut s = iced::widget::button::Style::default();

            // Estados de interacción sutiles
            let base = alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.08);
            let hover = alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.12);
            let press = alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.06);

            let bg = match status {
                iced::widget::button::Status::Hovered => hover,
                iced::widget::button::Status::Pressed => press,
                _ => base,
            };

            s.background = Some(Background::Color(bg));
            s.border = Border {
                color: alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.12),
                width: 1.0,
                radius: border::Radius::from(999.0),
            };
            s.text_color = t.foreground;
            s
        })
        .on_press(on_press)
        .into()
}

pub fn danger_button(_t: Tokens, label: String, on_press: Message) -> Element<'static, Message> {
    let danger = Color::from_rgba8(0xFF, 0x5A, 0x5A, 1.0);
    button(text(label).size(12).color(danger))
        .padding([6, 10])
        .style(move |_: &Theme, status| {
            let mut s = iced::widget::button::Style::default();
            let bg = match status {
                iced::widget::button::Status::Hovered => alpha(danger, 0.14),
                iced::widget::button::Status::Pressed => alpha(danger, 0.10),
                _ => alpha(danger, 0.08),
            };
            s.background = Some(Background::Color(bg));
            s.border = Border {
                color: alpha(danger, 0.22),
                width: 1.0,
                radius: border::Radius::from(999.0),
            };
            s.text_color = danger;
            s
        })
        .on_press(on_press)
        .into()
}

// Divisores

pub fn v_divider(t: Tokens) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fixed(1.0))
        .height(Length::Fill)
        .style(move |_| {
            // Usamos un color explícito para el borde
            container_style(alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.06), t.foreground)
        })
        .into()
}

pub fn h_divider(t: Tokens) -> Element<'static, Message> {
    container(Space::new())
        .width(Length::Fill)
        .height(Length::Fixed(1.0))
        .style(move |_| {
            container_style(alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.06), t.foreground)
        })
        .into()
}

/* ---------------- Sidebar ---------------- */

#[derive(Debug, Clone, Copy)]
enum NavKey {
    Overview,
    Workspaces,
    Universe,
    Forge,
    PmTools,
    Assets,
    Settings,
}

fn is_active(state: &AppState, key: NavKey) -> bool {
    match (key, &state.route) {
        (NavKey::Overview, Route::Overview) => true,
        (NavKey::Workspaces, Route::Workspaces) => true,
        (NavKey::Universe, Route::UniverseList)
        | (NavKey::Universe, Route::UniverseDetail { .. })
        | (NavKey::Universe, Route::Bestiary { .. })
        | (NavKey::Universe, Route::Timeline { .. }) => true,
        (NavKey::Forge, Route::Forge) => true,
        (NavKey::PmTools, Route::PmTools) => true,
        (NavKey::Assets, Route::Assets) => true,
        (NavKey::Settings, Route::Account) => true,
        _ => false,
    }
}

fn icon_for(key: NavKey) -> &'static str {
    match key {
        NavKey::Overview => "⌂",
        NavKey::Workspaces => "▦",
        NavKey::Universe => "◉",
        NavKey::Forge => "⚒",
        NavKey::PmTools => "≡",
        NavKey::Assets => "◫",
        NavKey::Settings => "⚙",
    }
}

fn group_label<'a>(label: &'a str, t: Tokens) -> E<'a> {
    container(text(label).size(10).color(alpha(t.muted_fg, 0.85)))
        .width(Length::Fill)
        .padding([10, 12])
        .into()
}

fn nav_button_style(
    t: Tokens,
    active: bool,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let mut s = iced::widget::button::Style::default();

    let bg = if active {
        alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.06)
    } else {
        match status {
            iced::widget::button::Status::Hovered => t.hover_bg,
            iced::widget::button::Status::Pressed => t.active_bg,
            _ => Color::TRANSPARENT,
        }
    };

    s.background = Some(Background::Color(bg));
    s.text_color = t.foreground;
    s.border = Border {
        color: if active {
            alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.08)
        } else {
            Color::TRANSPARENT
        },
        width: if active { 1.0 } else { 0.0 },
        radius: border::Radius::from(12.0),
    };

    s
}

fn nav_item<'a>(t: Tokens, label: &'a str, key: NavKey, on_press: Message, active: bool) -> E<'a> {
    let icon = text(icon_for(key))
        .size(14)
        .color(alpha(t.foreground, 0.92));

    let label = text(label)
        .size(14)
        .color(alpha(t.foreground, 0.92))
        .wrapping(iced::widget::text::Wrapping::None);

    let inner: Row<'a, Message> = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(container(icon).width(Length::Fixed(18.0)))
        .push(container(label).width(Length::Fill));

    button(container(inner).width(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fixed(36.0))
        .padding([0, 12])
        .style(move |_: &Theme, status| nav_button_style(t, active, status))
        .on_press(on_press)
        .into()
}

// =================================================================
// LA CORRECCIÓN CLAVE: El sidebar devuelve el panel + el borde
// =================================================================
pub fn sidebar<'a>(state: &'a AppState, t: Tokens) -> E<'a> {
    let mut modules: Column<'a, Message> = Column::new().spacing(30).width(Length::Fill);

    // Lista de módulos
    modules = modules
        .push(group_label("Modules", t))
        .push(nav_item(t, "Overview", NavKey::Overview, Message::Navigate(Route::Overview), is_active(state, NavKey::Overview)))
        .push(nav_item(t, "Workspaces", NavKey::Workspaces, Message::Navigate(Route::Workspaces), is_active(state, NavKey::Workspaces)))
        .push(nav_item(t, "Universe", NavKey::Universe, Message::Navigate(Route::UniverseList), is_active(state, NavKey::Universe)))
        .push(nav_item(t, "The Forge", NavKey::Forge, Message::Navigate(Route::Forge), is_active(state, NavKey::Forge)))
        .push(nav_item(t, "PM Tools", NavKey::PmTools, Message::Navigate(Route::PmTools), is_active(state, NavKey::PmTools)))
        .push(nav_item(t, "Assets", NavKey::Assets, Message::Navigate(Route::Assets), is_active(state, NavKey::Assets)));

    let mut account: Column<'a, Message> = Column::new().spacing(10).width(Length::Fill);
    account = account
        .push(group_label("Account", t))
        .push(nav_item(t, "Settings", NavKey::Settings, Message::Navigate(Route::Account), is_active(state, NavKey::Settings)));

    let inner_content: Column<'a, Message> = Column::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .push(modules)
        .push(container(Space::new()).height(Length::Fill)) // Spacer
        .push(account)
        .padding(Padding { top: 10.0, right: 10.0, bottom: 12.0, left: 10.0 })
        .spacing(10);

    // 1. Contenedor del Sidebar (Fondo oscuro, SIN BORDE propio)
    let panel = container(inner_content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| {
            let mut s = container_style(t.sidebar_bg, t.foreground);
            // IMPORTANTE: width 0.0 para no pintar bordes en los 4 lados
            s.border = Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: border::Radius::from(0.0),
            };
            s
        });

    // 2. Devolvemos una FILA que contiene: [ PANEL + DIVISOR ]
    // Esto simula un "Right Border" perfecto.
    Row::new()
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(0) // CERO espacio para que estén pegados
        .push(panel)
        .push(v_divider(t)) // La línea de 1px
        .into()
}

/* ---------------- Header ---------------- */

fn workspace_pill(t: Tokens, label: String) -> Element<'static, Message> {
    container(text(label).size(12).color(t.foreground))
        .padding([6, 12])
        .style(move |_| {
            let mut s = container_style(alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.06), t.foreground);
            s.border = Border {
                color: alpha(Color::from_rgba8(0xFF, 0xFF, 0xFF, 1.0), 0.10),
                width: 1.0,
                radius: border::Radius::from(999.0),
            };
            s
        })
        .into()
}

pub fn header<'a>(state: &'a AppState, t: Tokens) -> E<'a> {
    let brand = Column::new()
        .spacing(2)
        .push(text(APP_NAME).size(16).color(alpha(t.foreground, 0.92)))
        .push(text(APP_SLOGAN).size(12).color(alpha(t.muted_fg, 0.92)));

    let right = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(workspace_pill(t, state.active_workspace.clone()));

    let bar = Row::new()
        .align_y(Alignment::Center)
        .push(container(brand).width(Length::Fill))
        .push(right)
        .padding(Padding {
            top: 14.0,
            right: 24.0,
            bottom: 14.0,
            left: 24.0,
        });

    container(bar)
        .width(Length::Fill)
        .style(move |_| container_style(Color::TRANSPARENT, t.foreground))
        .into()
}
