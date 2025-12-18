use iced::{Element, Length, Theme};
use iced::widget::{container, scrollable, Column, Row};

use std::time::{Duration, Instant};

use crate::model::{Creature, Universe};
use crate::{pages, ui};

pub const APP_NAME: &str = "Titan Architect Studio";
pub const APP_ACRONYM: &str = "TAS";

pub fn run() -> iced::Result {
    iced::application(AppState::default, update, view)
        .title(title)
        .theme(app_theme)
        .run()
}

pub fn title(_state: &AppState) -> String {
    format!("{APP_NAME} ({APP_ACRONYM})")
}

pub fn app_theme(_state: &AppState) -> Theme {
    Theme::Dark
}

#[derive(Debug, Clone)]
pub enum Route {
    Overview,
    Workspaces,
    UniverseList,
    UniverseDetail { universe_id: String },
    Bestiary { universe_id: String },
    Timeline { universe_id: String },
    Forge,
    PmTools,
    Assets,
    Account,
}

impl Route {
    pub fn header_title(&self) -> &'static str {
        match self {
            Route::Overview => "Overview",
            Route::Workspaces => "Workspaces",
            Route::UniverseList
            | Route::UniverseDetail { .. }
            | Route::Bestiary { .. }
            | Route::Timeline { .. } => "Universe",
            Route::Forge => "The Forge",
            Route::PmTools => "PM Tools",
            Route::Assets => "Assets",
            Route::Account => "Account",
        }
    }
}

/// Temporary in-memory editor state for a creature.
///
/// NOTE: This is intentionally UI-only state. Persistence will come later.
#[derive(Debug, Clone)]
pub struct CreatureEditor {
    pub index: Option<usize>,

    pub name: String,
    pub kind: String,
    pub habitat: String,
    pub description: String,
    pub danger: String,
}

impl CreatureEditor {
    pub fn create_new() -> Self {
        Self {
            index: None,
            name: String::new(),
            kind: String::new(),
            habitat: String::new(),
            description: String::new(),
            danger: "Medium".to_string(),
        }
    }

    pub fn from_creature(index: usize, c: &Creature) -> Self {
        Self {
            index: Some(index),
            name: c.name.clone(),
            kind: c.kind.clone(),
            habitat: c.habitat.clone(),
            description: c.description.clone(),
            danger: c.danger.clone(),
        }
    }

    pub fn into_creature(self) -> Creature {
        Creature {
            name: self.name.trim().to_string(),
            kind: self.kind.trim().to_string(),
            habitat: self.habitat.trim().to_string(),
            description: self.description.trim().to_string(),
            danger: self.danger.trim().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Route),

    Logout,

    UniverseNameChanged(String),
    UniverseDescChanged(String),
    CreateUniverse,

    OpenUniverse(String),
    BackToUniverses,

    OpenBestiary(String),
    OpenTimeline(String),
    BackToUniverse(String),

    // Bestiary interactions
    BestiaryCardClicked(usize),

    CreatureEditorOpenCreate,
    CreatureEditorCancel,
    CreatureEditorSave,

    CreatureEditorNameChanged(String),
    CreatureEditorKindChanged(String),
    CreatureEditorHabitatChanged(String),
    CreatureEditorDescriptionChanged(String),
    CreatureEditorDangerChanged(String),
}

#[derive(Debug)]
pub struct AppState {
    pub route: Route,

    pub active_workspace: String,

    pub new_universe_name: String,
    pub new_universe_desc: String,

    pub universes: Vec<Universe>,
    pub creatures: Vec<Creature>,

    // Bestiary editor
    pub creature_editor: Option<CreatureEditor>,
    pub last_bestiary_click: Option<(usize, Instant)>,
}

impl Default for AppState {
    fn default() -> Self {
        let arhelis_id = "arhelis".to_string();

        Self {
            route: Route::Overview,
            active_workspace: "Arhelis".to_string(),

            new_universe_name: "".to_string(),
            new_universe_desc: "".to_string(),

            universes: vec![Universe {
                id: arhelis_id.clone(),
                name: "Arhelis".to_string(),
                description: "Fantasy Universe".to_string(),
                archived: false,
            }],

            creatures: vec![
                Creature {
                    name: "Fog Engulfer".to_string(),
                    kind: "Elemental · Relicto".to_string(),
                    habitat: "Pasos de montaña brumosos, valles cerrados al amanecer.".to_string(),
                    description: "Una criatura compuesta de vapor condensado y huesos de sus víctimas, solo sólida cuando ataca.".to_string(),
                    danger: "High".to_string(),
                },
                Creature {
                    name: "Oathbound Specter".to_string(),
                    kind: "Espectro · Constructo · Maldito".to_string(),
                    habitat: "Ruinas de castillos, antiguos campos de batalla, criptas de familias nobles.".to_string(),
                    description: "Una armadura vacía animada por el espíritu de un guerrero que murió rompiendo una promesa sagrada.".to_string(),
                    danger: "Medium".to_string(),
                },
                Creature {
                    name: "Obsidian Widow".to_string(),
                    kind: "Insectoide · Arácnido Acorazado".to_string(),
                    habitat: "Cavernas volcánicas, minas profundas y grietas montañosas.".to_string(),
                    description: "Una araña gigante con un caparazón cristalino que refleja la magia y garras afiladas como diamantes.".to_string(),
                    danger: "Extreme".to_string(),
                },
                Creature {
                    name: "Silt Siren".to_string(),
                    kind: "Anfibio · Dracónido Menor".to_string(),
                    habitat: "Pantanos, deltas de ríos y alcantarillas de grandes ciudades.".to_string(),
                    description: "Una criatura reptiliana que puede licuar su propia estructura ósea para esconderse en charcos poco profundos.".to_string(),
                    danger: "Medium".to_string(),
                },
                Creature {
                    name: "The Howling Mycophage".to_string(),
                    kind: "Híbrido · Necrófago · Bestia Infectada".to_string(),
                    habitat: "Bosques densos, cuevas húmedas y zonas con poca luz solar.".to_string(),
                    description: "Un lobo o bestia similar, mutado grotescamente por hongos parásitos que controlan su sistema nervioso.".to_string(),
                    danger: "High".to_string(),
                },
                Creature {
                    name: "The Vessel-Tearer".to_string(),
                    kind: "Monstruosidad · Mutante Arcano · Maldito".to_string(),
                    habitat: "Zonas de catástrofes mágicas recientes, grietas de maná abiertas, ruinas de laboratorios de hechiceros renegados.".to_string(),
                    description: "Una criatura grotescamente asimétrica cuya carne se ha fusionado con cristales de maná volátil, brillando con una luz enfermiza.".to_string(),
                    danger: "High".to_string(),
                },
            ],

            creature_editor: None,
            last_bestiary_click: None,
        }
    }
}

pub fn update(state: &mut AppState, message: Message) {
    const DOUBLE_CLICK_WINDOW: Duration = Duration::from_millis(420);

    match message {
        Message::Navigate(route) => state.route = route,

        Message::Logout => {
            // Offline app: route back to Overview for now
            state.route = Route::Overview;
        }

        Message::UniverseNameChanged(v) => state.new_universe_name = v,
        Message::UniverseDescChanged(v) => state.new_universe_desc = v,

        Message::CreateUniverse => {
            let name = state.new_universe_name.trim().to_string();
            if name.is_empty() {
                return;
            }

            let id = name.to_lowercase().replace(' ', "-");

            state.universes.push(Universe {
                id,
                name,
                description: state.new_universe_desc.trim().to_string(),
                archived: false,
            });

            state.new_universe_name.clear();
            state.new_universe_desc.clear();
        }

        Message::OpenUniverse(id) => state.route = Route::UniverseDetail { universe_id: id },
        Message::BackToUniverses => state.route = Route::UniverseList,

        Message::OpenBestiary(id) => {
            state.creature_editor = None;
            state.last_bestiary_click = None;
            state.route = Route::Bestiary { universe_id: id };
        }
        Message::OpenTimeline(id) => state.route = Route::Timeline { universe_id: id },
        Message::BackToUniverse(id) => {
            state.creature_editor = None;
            state.last_bestiary_click = None;
            state.route = Route::UniverseDetail { universe_id: id };
        }

        // Bestiary interactions
        Message::BestiaryCardClicked(index) => {
            let now = Instant::now();

            match state.last_bestiary_click.take() {
                Some((i, t)) if i == index && now.duration_since(t) <= DOUBLE_CLICK_WINDOW => {
                    if let Some(c) = state.creatures.get(index) {
                        state.creature_editor = Some(CreatureEditor::from_creature(index, c));
                    }
                }
                _ => {
                    state.last_bestiary_click = Some((index, now));
                }
            }
        }

        Message::CreatureEditorOpenCreate => {
            state.creature_editor = Some(CreatureEditor::create_new());
            state.last_bestiary_click = None;
        }

        Message::CreatureEditorCancel => {
            state.creature_editor = None;
            state.last_bestiary_click = None;
        }

        Message::CreatureEditorSave => {
            let Some(editor) = state.creature_editor.take() else {
                return;
            };

            // Minimal validation: require name
            if editor.name.trim().is_empty() {
                state.creature_editor = Some(editor);
                return;
            }

            let idx = editor.index;
            let updated = editor.into_creature();

            match idx {
                Some(i) if i < state.creatures.len() => state.creatures[i] = updated,
                _ => state.creatures.push(updated),
            }

            state.last_bestiary_click = None;
        }

        Message::CreatureEditorNameChanged(v) => {
            if let Some(ed) = state.creature_editor.as_mut() {
                ed.name = v;
            }
        }
        Message::CreatureEditorKindChanged(v) => {
            if let Some(ed) = state.creature_editor.as_mut() {
                ed.kind = v;
            }
        }
        Message::CreatureEditorHabitatChanged(v) => {
            if let Some(ed) = state.creature_editor.as_mut() {
                ed.habitat = v;
            }
        }
        Message::CreatureEditorDescriptionChanged(v) => {
            if let Some(ed) = state.creature_editor.as_mut() {
                ed.description = v;
            }
        }
        Message::CreatureEditorDangerChanged(v) => {
            if let Some(ed) = state.creature_editor.as_mut() {
                ed.danger = v;
            }
        }
    }
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let t = ui::Tokens::nub_dark();

    let sidebar = ui::sidebar(state, t);
    let header = ui::header(state, t);

    let page: Element<'_, Message> = match &state.route {
        Route::Overview => pages::overview(state, t),
        Route::Workspaces => pages::workspaces_stub(state, t),
        Route::UniverseList => pages::universe_list(state, t),
        Route::UniverseDetail { universe_id } => pages::universe_detail(state, t, universe_id),
        Route::Bestiary { universe_id } => pages::bestiary(state, t, universe_id),
        Route::Timeline { universe_id } => pages::timeline_stub(state, t, universe_id),
        Route::Forge => pages::forge_stub(state, t),
        Route::PmTools => pages::pm_stub(state, t),
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
        .spacing(18)
        .push(container(sidebar).width(Length::Fixed(86.0)).height(Length::Fill))
        .push(ui::v_divider(t))
        .push(right)
        .width(Length::Fill)
        .height(Length::Fill);

    ui::shell(t, root.into())
}
