use std::time::Instant;
use std::collections::{HashSet, VecDeque};
use iced::widget::text_editor; // Importante para el editor

use crate::model::{
    Creature, Universe, Card, KanbanBoardData, Board, Location, TimelineEvent, TimelineEra, Project, UniverseSnapshot,
    Story, Scene
};
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

#[derive(Debug, Clone, PartialEq)]
pub enum DbAction {
    CreateUniverse(String, String),
    DeleteUniverse(String),
    InjectDemoData(String),
    ResetDemoDataScoped(String, DemoResetScope),

    SnapshotCreate { universe_id: String, name: String },
    SnapshotDelete { snapshot_id: String },
    SnapshotRestore { snapshot_id: String },

    CreateBoard(String),
    DeleteBoard(String),

    SaveCreature(Creature, String),
    ArchiveCreature(String, bool),
    DeleteCreature(String),

    SaveLocation(Location),
    DeleteLocation(String),

    SaveEvent(TimelineEvent),
    DeleteEvent(String),
    SaveEra(TimelineEra),
    DeleteEra(String),

    SaveCard(Card),
    MoveCard(String, String, f64),
    RebalanceColumn(String),
    DeleteCard(String),

    // --- THE FORGE ACTIONS ---
    CreateStory(String, String),
    UpdateStory(Story),
    DeleteStory(String),

    CreateScene(String, String),
    UpdateScene(Scene),
    DeleteScene(String),
    ReorderScene(String, i64),
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

    pub active_project: Option<Project>,
    pub projects: Vec<Project>,
    pub is_creating_project: bool,
    pub new_project_name: String,

    pub universes: Vec<Universe>,
    pub new_universe_name: String,
    pub new_universe_desc: String,

    pub pending_demo_reset: Option<(String, DemoResetScope)>,

    // Collapsible dev panel (Arhelis only for now)
    pub dev_panel_open: bool,

    // Debug overlay
    pub debug_overlay_open: bool,
    pub debug_schema_version: Option<i64>,

    // Snapshots
    pub snapshot_name: String,
    pub snapshots: Vec<UniverseSnapshot>,

    // Integrity
    pub integrity_issues: Vec<String>,
    pub integrity_busy: bool,

    // Cache flags
    pub loaded_creatures_universe: Option<String>,
    pub loaded_locations_universe: Option<String>,
    pub loaded_timeline_universe: Option<String>,
    pub loaded_snapshots_universe: Option<String>,
    pub loaded_forge_universe: Option<String>,

    pub data_dirty: bool,

    pub creatures: Vec<Creature>,
    pub locations: Vec<Location>,
    pub timeline_events: Vec<TimelineEvent>,
    pub timeline_eras: Vec<TimelineEra>,

    // --- THE FORGE STATE ---
    pub stories: Vec<Story>,
    pub active_story_id: Option<String>,
    pub active_story_scenes: Vec<Scene>,
    pub active_scene_id: Option<String>,
    pub forge_content: text_editor::Content, // <--- EL ESTADO DEL EDITOR VISUAL

    pub boards_list: Vec<Board>,
    pub new_board_name: String,
    pub pm_state: PmState,
    pub pm_data: Option<KanbanBoardData>,
    pub hovered_column: Option<String>,
    pub hovered_card: Option<String>,
    pub last_pm_click: Option<(String, Instant)>,

    pub creature_editor: Option<CreatureEditor>,
    pub last_bestiary_click: Option<(usize, Instant)>,

    pub location_editor: Option<LocationEditor>,
    pub last_location_click: Option<(String, Instant)>,
    pub expanded_locations: HashSet<String>,
    pub selected_location: Option<String>,

    pub event_editor: Option<EventEditor>,
    pub era_editor: Option<EraEditor>,
    pub last_timeline_click: Option<(String, Instant)>,

    pub db_queue: VecDeque<DbAction>,
    pub db_inflight: Option<DbAction>,

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

            dev_panel_open: true,

            debug_overlay_open: false,
            debug_schema_version: None,

            snapshot_name: String::new(),
            snapshots: vec![],

            integrity_issues: vec![],
            integrity_busy: false,

            loaded_creatures_universe: None,
            loaded_locations_universe: None,
            loaded_timeline_universe: None,
            loaded_snapshots_universe: None,
            loaded_forge_universe: None,

            data_dirty: false,

            creatures: vec![],
            locations: vec![],
            timeline_events: vec![],
            timeline_eras: vec![],

            // Forge Init
            stories: vec![],
            active_story_id: None,
            active_story_scenes: vec![],
            active_scene_id: None,
            forge_content: text_editor::Content::new(),

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

            toasts: vec![],
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