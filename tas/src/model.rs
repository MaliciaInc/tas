use sqlx::FromRow;
use std::fmt;
use serde::{Serialize, Deserialize};

// --- PROJECTS (WORKSPACES) ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub last_opened: chrono::DateTime<chrono::Local>,
    pub created_at: chrono::DateTime<chrono::Local>,
}

// --- UNIVERSE & BESTIARY ---
#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Universe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub archived: bool,
}

#[derive(Debug, Clone, FromRow, PartialEq)] // <--- FIX: Agregado PartialEq
pub struct Creature {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub habitat: String,
    pub description: String,
    pub danger: String,
    pub home_location_id: Option<String>,
    #[sqlx(default)]
    pub archived: bool,
}

// --- LOCATIONS ---
#[derive(Debug, Clone, FromRow, PartialEq)] // <--- FIX: Agregado PartialEq
pub struct Location {
    pub id: String,
    pub universe_id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub description: String,
    pub kind: String,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

// --- TIMELINE ---
#[derive(Debug, Clone, FromRow, PartialEq)] // <--- FIX: Agregado PartialEq
pub struct TimelineEra {
    pub id: String,
    pub universe_id: String,
    pub name: String,
    pub start_year: i64,
    pub end_year: Option<i64>,
    pub description: String,
    pub color: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)] // <--- FIX: Agregado PartialEq
pub struct TimelineEvent {
    pub id: String,
    pub universe_id: String,
    pub title: String,
    pub description: String,
    pub year: i64,
    pub display_date: String,
    pub importance: String,
    pub kind: String,
    pub color: String,
    pub location_id: Option<String>,
}

// --- PM TOOLS (KANBAN) ---

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct Board {
    pub id: String,
    pub name: String,
    #[allow(dead_code)] pub kind: String,
}

#[derive(Debug, Clone, FromRow, PartialEq)]
pub struct BoardColumn {
    pub id: String,
    #[allow(dead_code)] pub board_id: String,
    pub name: String,
    pub position: i32,
}

#[derive(Debug, Clone, FromRow, PartialEq)] // <--- FIX: Agregado PartialEq
pub struct Card {
    pub id: String,
    pub column_id: String,
    pub title: String,
    pub description: String,
    pub position: f64,
    #[sqlx(default)] pub priority: String,
}

#[derive(Debug, Clone)]
pub struct KanbanBoardData {
    pub board: Board,
    pub columns: Vec<(BoardColumn, Vec<Card>)>,
}