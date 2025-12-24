use iced::widget::text_editor;
use crate::app::Route;
use crate::model::{Creature, Universe, Card, KanbanBoardData, Board, Location, TimelineEvent, TimelineEra, Project};
use crate::state::DemoResetScope;

#[derive(Debug, Clone)]
pub enum PmMessage {
    BoardNameChanged(String), CreateBoard, DeleteBoard(String), OpenBoard(String),
    BoardLoaded(KanbanBoardData), DragStart(Card), ColumnHovered(String), CardHovered(String),
    OpenCreate(String), OpenGlobalCreate, OpenEdit(Card), TitleChanged(String),
    DescChanged(text_editor::Action), PriorityChanged(String), Save, Delete, Cancel,
}

#[derive(Debug, Clone)]
pub enum BestiaryMessage {
    Open(String), CardClicked(usize), EditorOpenCreate, EditorCancel, EditorSave,
    NameChanged(String), KindChanged(String), HabitatChanged(String),
    DescriptionChanged(text_editor::Action), DangerChanged(String), LocationChanged(Option<Location>),
    Delete(String), Archive(String), Restore(String),
}

#[derive(Debug, Clone)]
pub enum UniverseMessage {
    NameChanged(String), DescChanged(String), Create, Delete(String), Open(String),
    InjectDemoData(String),

    // Reset demo data (Arhelis-only) - 2-step UI confirm
    ResetDemoPrompt(String, DemoResetScope),
    ResetDemoConfirm,
    ResetDemoCancel,
}

#[derive(Debug, Clone)]
pub enum LocationsMessage {
    Open(String), EditorOpenCreate(Option<String>), CardClicked(String), EditorCancel, EditorSave,
    Delete(String), NameChanged(String), KindChanged(String), DescriptionChanged(text_editor::Action),
    ToggleExpand(String), Select(String), CardDoubleClicked(String),
}

#[derive(Debug, Clone)]
pub enum TimelineMessage {
    Open(String),
    EditorOpenCreateEvent(Option<i64>), EditorOpenCreateEra,
    EditEvent(String), EditEra(String),
    CardClicked(String), EraBannerClicked(String),
    EditorCancel, EditorSaveEvent, EditorSaveEra,
    DeleteEvent(String), DeleteEra(String),
    TitleChanged(String), YearChanged(String), DisplayDateChanged(String), ImportanceChanged(String),
    KindChanged(String), ColorChanged(String), LocationChanged(Option<Location>), DescriptionChanged(text_editor::Action),
    EraNameChanged(String), EraStartChanged(String), EraEndChanged(String), EraColorChanged(String), EraDescChanged(text_editor::Action),
}

#[derive(Debug, Clone)]
pub enum WorkspaceMessage {
    CreateStart, CreateCancel, NameChanged(String), CreateConfirm,
    Open(String), CloseProject, RefreshList,
    Delete(String),
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(Route), MouseMoved(iced::Point), MouseReleased,
    Tick(std::time::Instant),
    ToastDismiss(u64),

    Pm(PmMessage), Bestiary(BestiaryMessage), Universe(UniverseMessage), Locations(LocationsMessage),
    Timeline(TimelineMessage), Workspace(WorkspaceMessage),

    UniversesFetched(Result<Vec<Universe>, String>),
    CreaturesFetched(Result<Vec<Creature>, String>),
    BoardsFetched(Result<Vec<Board>, String>),
    PmBoardFetched(Result<KanbanBoardData, String>),
    LocationsFetched(Result<Vec<Location>, String>),
    TimelineFetched(Result<(Vec<TimelineEvent>, Vec<TimelineEra>), String>),

    ProjectsLoaded(Vec<Project>),
    ProjectCreated(Result<Project, String>),
    DbLoaded(Result<crate::db::Database, String>),
    ActionDone(Result<(), String>),
    GlobalEvent(iced::Event),

    BackToUniverses, BackToUniverse(String), OpenTimeline(String), GoToLocation(String, String),
}
