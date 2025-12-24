use std::time::Instant;
use std::collections::{HashSet, VecDeque};
use crate::model::{Creature, Universe, Card, KanbanBoardData, Board, Location, TimelineEvent, TimelineEra, Project};
use crate::app::{Route, PmState};
use crate::editors::{CreatureEditor, LocationEditor, EventEditor, EraEditor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoResetScope {
    All,
    Timeline,
    Locations,
    Bestiary,
    PmTools,
}

// --- ACTION QUEUE SYSTEM ---
#[derive(Debug, Clone, PartialEq)]
pub enum DbAction {
    // Universe
    CreateUniverse(String, String),
    DeleteUniverse(String),
    InjectDemoData(String),

    ResetDemoDataScoped(String, DemoResetScope),

    // Project
    CreateBoard(String),
    DeleteBoard(String),

    // Bestiary
    SaveCreature(Creature, String),
    ArchiveCreature(String, bool),
    DeleteCreature(String),

    // Locations
    SaveLocation(Location),
    DeleteLocation(String),

    // Timeline
    SaveEvent(TimelineEvent),
    DeleteEvent(String),
    SaveEra(TimelineEra),
    DeleteEra(String),

    // PM / Kanban
    SaveCard(Card),
    MoveCard(String, String, f64),
    DeleteCard(String),
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub kind: ToastKind,
    pub created_at: Instant,
    pub ttl_secs: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastKind { Info, Success, Error }

#[derive(Debug)]
pub struct AppState {
    pub route: Route,

    // PROJECT / WORKSPACE STATE
    pub active_project: Option<Project>,
    pub projects: Vec<Project>,
    pub is_creating_project: bool,
    pub new_project_name: String,

    // DATA CACHE
    pub universes: Vec<Universe>,
    pub new_universe_name: String,
    pub new_universe_desc: String,

    // Demo reset confirmation (Arhelis-only)
    pub pending_demo_reset: Option<(String, DemoResetScope)>,

    // FLAGS DE CACHÉ
    pub loaded_creatures_universe: Option<String>,
    pub loaded_locations_universe: Option<String>,
    pub loaded_timeline_universe: Option<String>,

    pub data_dirty: bool,

    pub creatures: Vec<Creature>,
    pub locations: Vec<Location>,
    pub timeline_events: Vec<TimelineEvent>,
    pub timeline_eras: Vec<TimelineEra>,

    // PM STATE
    pub boards_list: Vec<Board>,
    pub new_board_name: String,
    pub pm_state: PmState,
    pub pm_data: Option<KanbanBoardData>,
    pub hovered_column: Option<String>,
    pub hovered_card: Option<String>,
    pub last_pm_click: Option<(String, Instant)>,

    // EDITORS & UI STATE
    pub creature_editor: Option<CreatureEditor>,
    pub last_bestiary_click: Option<(usize, Instant)>,

    pub location_editor: Option<LocationEditor>,
    pub last_location_click: Option<(String, Instant)>,
    pub expanded_locations: HashSet<String>,
    pub selected_location: Option<String>,

    pub event_editor: Option<EventEditor>,
    pub era_editor: Option<EraEditor>,
    pub last_timeline_click: Option<(String, Instant)>,

    // QUEUE SYSTEM
    pub db_queue: VecDeque<DbAction>,
    pub db_inflight: Option<DbAction>,

    // NOTIFICATIONS
    pub toasts: Vec<Toast>,
    pub toast_counter: u64,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            route: Route::Overview,

            active_project: None,
            projects: vec![],
            is_creating_project: false,
            new_project_name: String::new(),

            universes: vec![],
            new_universe_name: String::new(),
            new_universe_desc: String::new(),

            pending_demo_reset: None,

            creatures: vec![],
            loaded_creatures_universe: None,
            locations: vec![],
            loaded_locations_universe: None,
            loaded_timeline_universe: None,
            data_dirty: false,

            timeline_events: vec![],
            timeline_eras: vec![],

            boards_list: vec![],
            new_board_name: String::new(),

            pm_state: PmState::Idle,
            pm_data: None,
            hovered_column: None,
            hovered_card: None,
            last_pm_click: None,

            creature_editor: None,
            last_bestiary_click: None,

            location_editor: None,
            last_location_click: None,
            expanded_locations: HashSet::new(),
            selected_location: None,

            event_editor: None,
            era_editor: None,
            last_timeline_click: None,

            db_queue: VecDeque::new(),
            db_inflight: None,

            toasts: Vec::new(),
            toast_counter: 0,
        }
    }
}

impl AppState {
    pub fn queue(&mut self, action: DbAction) {
        self.db_queue.push_back(action);
    }

    pub fn show_toast(&mut self, msg: impl Into<String>, kind: ToastKind) {
        self.toast_counter += 1;
        self.toasts.push(Toast {
            id: self.toast_counter,
            message: msg.into(),
            kind,
            created_at: Instant::now(),
            ttl_secs: 4,
        });
    }
}
