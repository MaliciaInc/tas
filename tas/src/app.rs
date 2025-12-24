use iced::{Element, Length, Point, Theme, Vector, Color};
use iced::widget::{container, scrollable, text, Column, Row, Stack};
use crate::model::Card;
use crate::{pages, ui};

// --- RE-EXPORTS ---
pub use crate::state::AppState;
pub use crate::messages::{
    Message, PmMessage, BestiaryMessage, UniverseMessage, LocationsMessage, TimelineMessage, WorkspaceMessage
};
pub use crate::editors::{CreatureEditor, LocationEditor, EventEditor, EraEditor};

pub const APP_NAME: &str = "Titan Architect Studio";
pub const APP_ACRONYM: &str = "TAS";

pub fn app_theme(_state: &AppState) -> Theme { Theme::Dark }

#[derive(Debug, Clone)]
pub enum Route {
    Overview, Workspaces, UniverseList,
    UniverseDetail { universe_id: String },
    Bestiary { universe_id: String },
    Locations { universe_id: String },
    Timeline { universe_id: String },
    PmList, PmBoard { board_id: String },
    Forge, Assets, Account,
}
impl Route { #[allow(dead_code)] pub fn header_title(&self) -> &'static str { "" } }

#[derive(Debug, Clone)]
pub enum PmState {
    Idle,
    Dragging { card: Card, original_col: String, drag_start: Point, current_cursor: Point, active: bool },
    Editing { card_id: Option<String>, column_id: String, title: String, description: iced::widget::text_editor::Content, priority: String },
}
impl Default for PmState { fn default() -> Self { Self::Idle } }

// --- VIEW DISPATCHER ---
pub fn view(state: &AppState) -> Element<'_, Message> {
    let t = ui::Tokens::nub_dark();

    // 1. LAUNCHER MODE
    if state.active_project.is_none() {
        return pages::launcher::launcher_view(state, t);
    }

    // 2. STUDIO MODE
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
        Route::Forge => pages::forge_stub(state, t),
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

    // OVERLAYS (Modales)
    if let PmState::Editing { title, description, priority, card_id, .. } = &state.pm_state {
        let is_new = card_id.is_none();
        stack = stack.push(crate::pages::pm_board::render_modal(t, title, description, priority, is_new));
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
    if let PmState::Dragging { card, current_cursor, active, .. } = &state.pm_state {
        if *active {
            let ghost = container(text(&card.title).size(14).style(move |_| iced::widget::text::Style { color: Some(t.foreground) }))
                .padding(12).width(Length::Fixed(280.0)).style(move |_: &Theme| {
                let mut s = ui::container_style(t.shell_b, t.foreground);
                s.border.color = t.accent; s.border.width = 2.0; s.border.radius = 6.0.into();
                s.background = Some(ui::alpha(t.background, 0.9).into());
                s.shadow = iced::Shadow { color: Color::BLACK, offset: Vector::new(0.0, 10.0), blur_radius: 20.0 };
                s
            });
            stack = stack.push(container(ghost).padding(iced::Padding {
                top: current_cursor.y + 10.0, left: current_cursor.x + 10.0, bottom: 0.0, right: 0.0
            }));
        }
    }

    // BULLDOZER: TOASTS LAYER (Siempre encima)
    if !state.toasts.is_empty() {
        stack = stack.push(ui::toasts_overlay(t, &state.toasts));
    }

    stack.into()
}