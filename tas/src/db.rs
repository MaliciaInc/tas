use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;

use crate::model::{
    Board, BoardColumn, Card, Creature, KanbanBoardData, Location, TimelineEvent, TimelineEra, Universe,
};
use crate::state::DemoResetScope;

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn connect(db_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        crate::logger::info(&format!("Database connecting to: {:?}", db_path));

        let options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;

        // Schema versioning + sequential migrations
        crate::db_migrations::apply(&pool).await?;

        let db = Self { pool };

        // Canonical board integrity
        db.repair_integrity().await?;

        // Always ensure demo baseline exists
        crate::db_seed::run_all(&db.pool, "u-arhelis-01").await?;

        Ok(db)
    }

    async fn repair_integrity(&self) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO universes (id, name, description) VALUES \
             ('u-arhelis-01', 'Arhelis', 'Un mundo fracturado por la magia antigua.')",
        )
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO boards (id, name, kind) VALUES \
             ('board-main', 'Development Roadmap', 'kanban')",
        )
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "INSERT OR IGNORE INTO board_columns (id, board_id, name, position) VALUES \
             ('col-hold', 'board-main', 'On-Hold', 0), \
             ('col-todo', 'board-main', 'To Do', 1), \
             ('col-progress', 'board-main', 'In Progress', 2), \
             ('col-done', 'board-main', 'Done', 3)",
        )
            .execute(&self.pool)
            .await?;

        #[derive(sqlx::FromRow)]
        struct ColRow { id: String, name: String }

        let cols: Vec<ColRow> =
            sqlx::query_as("SELECT id, name FROM board_columns WHERE board_id='board-main'")
                .fetch_all(&self.pool)
                .await?;

        fn canonical_id_for(name: &str) -> Option<&'static str> {
            match name.trim() {
                "On-Hold" | "On Hold" => Some("col-hold"),
                "To Do" => Some("col-todo"),
                "In Progress" => Some("col-progress"),
                "Done" => Some("col-done"),
                _ => None,
            }
        }

        for c in &cols {
            let Some(target) = canonical_id_for(&c.name) else { continue };
            if c.id == target { continue; }
            sqlx::query("UPDATE cards SET column_id = ? WHERE column_id = ?")
                .bind(target)
                .bind(&c.id)
                .execute(&self.pool)
                .await?;
        }

        for c in &cols {
            let is_canonical = matches!(c.id.as_str(), "col-hold" | "col-todo" | "col-progress" | "col-done");
            if !is_canonical {
                sqlx::query("DELETE FROM board_columns WHERE id = ? AND board_id='board-main'")
                    .bind(&c.id)
                    .execute(&self.pool)
                    .await?;
            }
        }

        sqlx::query("UPDATE board_columns SET position=0 WHERE id='col-hold' AND board_id='board-main'")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE board_columns SET position=1 WHERE id='col-todo' AND board_id='board-main'")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE board_columns SET position=2 WHERE id='col-progress' AND board_id='board-main'")
            .execute(&self.pool)
            .await?;
        sqlx::query("UPDATE board_columns SET position=3 WHERE id='col-done' AND board_id='board-main'")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // --- Demo helpers ---
    pub async fn inject_demo_data(&self, universe_id: String) -> Result<(), sqlx::Error> {
        if universe_id == "u-arhelis-01" {
            crate::db_seed::run_all(&self.pool, &universe_id).await
        } else {
            Ok(())
        }
    }

    pub async fn reset_demo_data_scoped(&self, universe_id: String, scope: DemoResetScope) -> Result<(), sqlx::Error> {
        if universe_id != "u-arhelis-01" {
            return Ok(());
        }

        let mut tx = self.pool.begin().await?;

        match scope {
            DemoResetScope::All => {
                // Timeline
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;

                // Bestiary
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;

                // Locations (and null-out bestiary references in case older DBs lack FK constraints)
                sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;

                // PM tools (system board cards only)
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')")
                    .execute(&mut *tx).await?;
            }

            DemoResetScope::Timeline => {
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
            }

            DemoResetScope::Locations => {
                sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
            }

            DemoResetScope::Bestiary => {
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?")
                    .bind(&universe_id).execute(&mut *tx).await?;
            }

            DemoResetScope::PmTools => {
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')")
                    .execute(&mut *tx).await?;
            }
        }

        tx.commit().await?;

        // Rebuild known-good invariants + reseed only what applies
        self.repair_integrity().await?;

        match scope {
            DemoResetScope::All => {
                crate::db_seed::run_all(&self.pool, &universe_id).await?;
            }
            DemoResetScope::Timeline => {
                crate::db_seed::timeline::seed(&self.pool, &universe_id).await?;
            }
            DemoResetScope::Locations => {
                crate::db_seed::locations::seed(&self.pool, &universe_id).await?;
            }
            DemoResetScope::Bestiary => {
                crate::db_seed::bestiary::seed(&self.pool, &universe_id).await?;
            }
            DemoResetScope::PmTools => {
                crate::db_seed::pm_tools::seed(&self.pool).await?;
            }
        }

        Ok(())
    }

    // --- Boards API ---
    async fn create_board_internal(&self, name: String, force_id: Option<String>) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let bid = force_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        sqlx::query("INSERT INTO boards (id, name, kind) VALUES (?, ?, 'kanban')")
            .bind(&bid).bind(name).execute(&mut *tx).await?;

        let cols = vec![("On-Hold", 0), ("To Do", 1), ("In Progress", 2), ("Done", 3)];
        for (cname, pos) in cols {
            let cid = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO board_columns (id, board_id, name, position) VALUES (?, ?, ?, ?)")
                .bind(cid).bind(&bid).bind(cname).bind(pos).execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn create_board(&self, name: String) -> Result<(), sqlx::Error> {
        self.create_board_internal(name, None).await
    }

    pub async fn delete_board(&self, board_id: String) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;

        let cols: Vec<(String,)> = sqlx::query_as("SELECT id FROM board_columns WHERE board_id = ?")
            .bind(&board_id).fetch_all(&mut *tx).await?;

        for (cid,) in cols {
            sqlx::query("DELETE FROM cards WHERE column_id = ?")
                .bind(&cid).execute(&mut *tx).await?;
        }

        sqlx::query("DELETE FROM board_columns WHERE board_id = ?")
            .bind(&board_id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM boards WHERE id = ?")
            .bind(&board_id).execute(&mut *tx).await?;

        tx.commit().await?;
        Ok(())
    }

    // --- Universe API ---
    pub async fn get_all_universes(&self) -> Result<Vec<Universe>, sqlx::Error> {
        sqlx::query_as::<_, Universe>(
            "SELECT id, name, description, archived FROM universes ORDER BY created_at DESC",
        )
            .fetch_all(&self.pool).await
    }

    pub async fn create_universe(&self, name: String, description: String) -> Result<(), sqlx::Error> {
        let uid = format!("u-{}", Uuid::new_v4());
        sqlx::query("INSERT INTO universes (id, name, description) VALUES (?, ?, ?)")
            .bind(uid).bind(name).bind(description).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_universe(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("PRAGMA foreign_keys = ON").execute(&self.pool).await?;
        sqlx::query("DELETE FROM universes WHERE id = ?")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // --- Bestiary ---
    pub async fn get_creatures(&self, universe_id: String) -> Result<Vec<Creature>, sqlx::Error> {
        sqlx::query_as::<_, Creature>(
            "SELECT id, name, kind, habitat, description, danger, home_location_id, archived \
             FROM bestiary_entries WHERE universe_id = ? ORDER BY archived ASC, created_at DESC",
        )
            .bind(universe_id).fetch_all(&self.pool).await
    }

    pub async fn upsert_creature(&self, c: Creature, universe_id: String) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO bestiary_entries
               (id, universe_id, name, kind, habitat, description, danger, home_location_id, archived, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
               ON CONFLICT(id) DO UPDATE SET
               name=excluded.name,
               kind=excluded.kind,
               habitat=excluded.habitat,
               description=excluded.description,
               danger=excluded.danger,
               home_location_id=excluded.home_location_id,
               archived=excluded.archived,
               updated_at=datetime('now')"#,
        )
            .bind(c.id).bind(universe_id).bind(c.name).bind(c.kind).bind(c.habitat)
            .bind(c.description).bind(c.danger).bind(c.home_location_id).bind(c.archived)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn set_creature_archived(&self, id: String, archived: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bestiary_entries SET archived = ? WHERE id = ?")
            .bind(archived).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_creature(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bestiary_entries WHERE id = ?")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // --- Locations ---
    pub async fn get_locations_flat(&self, universe_id: String) -> Result<Vec<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, universe_id, parent_id, name, description, kind \
             FROM locations WHERE universe_id = ? ORDER BY name ASC",
        )
            .bind(universe_id).fetch_all(&self.pool).await
    }

    pub async fn upsert_location(&self, l: Location) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO locations
               (id, universe_id, parent_id, name, description, kind, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
               ON CONFLICT(id) DO UPDATE SET
               parent_id=excluded.parent_id,
               name=excluded.name,
               description=excluded.description,
               kind=excluded.kind,
               updated_at=datetime('now')"#,
        )
            .bind(l.id).bind(l.universe_id).bind(l.parent_id).bind(l.name)
            .bind(l.description).bind(l.kind).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_location(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM locations WHERE id = ?")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // --- Timeline ---
    pub async fn get_timeline_events(&self, universe_id: String) -> Result<Vec<TimelineEvent>, sqlx::Error> {
        sqlx::query_as::<_, TimelineEvent>(
            "SELECT id, universe_id, title, description, year, COALESCE(display_date, '') AS display_date, \
                    importance, kind, color, location_id \
             FROM timeline_events WHERE universe_id = ? ORDER BY year ASC",
        )
            .bind(universe_id).fetch_all(&self.pool).await
    }

    pub async fn get_timeline_eras(&self, universe_id: String) -> Result<Vec<TimelineEra>, sqlx::Error> {
        sqlx::query_as::<_, TimelineEra>(
            "SELECT id, universe_id, name, start_year, end_year, description, color \
             FROM timeline_eras WHERE universe_id = ? ORDER BY start_year ASC",
        )
            .bind(universe_id).fetch_all(&self.pool).await
    }

    pub async fn upsert_timeline_event(&self, e: TimelineEvent) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO timeline_events
               (id, universe_id, title, description, year, display_date, importance, kind, color, location_id, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
               ON CONFLICT(id) DO UPDATE SET
               title=excluded.title,
               description=excluded.description,
               year=excluded.year,
               display_date=excluded.display_date,
               importance=excluded.importance,
               kind=excluded.kind,
               color=excluded.color,
               location_id=excluded.location_id,
               updated_at=datetime('now')"#,
        )
            .bind(e.id).bind(e.universe_id).bind(e.title).bind(e.description)
            .bind(e.year).bind(e.display_date).bind(e.importance).bind(e.kind)
            .bind(e.color).bind(e.location_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_timeline_event(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM timeline_events WHERE id = ?")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn upsert_timeline_era(&self, e: TimelineEra) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO timeline_eras
               (id, universe_id, name, start_year, end_year, description, color, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now'))
               ON CONFLICT(id) DO UPDATE SET
               name=excluded.name,
               start_year=excluded.start_year,
               end_year=excluded.end_year,
               description=excluded.description,
               color=excluded.color"#,
        )
            .bind(e.id).bind(e.universe_id).bind(e.name).bind(e.start_year)
            .bind(e.end_year).bind(e.description).bind(e.color).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_timeline_era(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM timeline_eras WHERE id = ?")
            .bind(id).execute(&self.pool).await?;
        Ok(())
    }

    // --- Kanban ---
    pub async fn get_all_boards(&self) -> Result<Vec<Board>, sqlx::Error> {
        sqlx::query_as::<_, Board>("SELECT id, name, kind FROM boards ORDER BY created_at DESC")
            .fetch_all(&self.pool).await
    }

    pub async fn get_kanban_data(&self, board_id: String) -> Result<KanbanBoardData, sqlx::Error> {
        let board = sqlx::query_as::<_, Board>("SELECT id, name, kind FROM boards WHERE id = ?")
            .bind(&board_id).fetch_one(&self.pool).await?;

        let columns_raw = sqlx::query_as::<_, BoardColumn>(
            "SELECT id, board_id, name, position FROM board_columns WHERE board_id = ? ORDER BY position ASC",
        )
            .bind(&board_id).fetch_all(&self.pool).await?;

        let mut columns_data = Vec::new();
        for col in columns_raw {
            let cards = sqlx::query_as::<_, Card>(
                "SELECT id, column_id, title, description, position, priority FROM cards WHERE column_id = ? ORDER BY position ASC",
            )
                .bind(&col.id).fetch_all(&self.pool).await?;
            columns_data.push((col, cards));
        }

        Ok(KanbanBoardData { board, columns: columns_data })
    }

    pub async fn upsert_card(&self, c: Card) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"INSERT INTO cards (id, column_id, title, description, position, priority, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
               ON CONFLICT(id) DO UPDATE SET
               column_id=excluded.column_id,
               title=excluded.title,
               description=excluded.description,
               position=excluded.position,
               priority=excluded.priority,
               updated_at=datetime('now')"#,
        )
            .bind(c.id).bind(c.column_id).bind(c.title).bind(c.description)
            .bind(c.position).bind(c.priority).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn move_card(&self, card_id: String, new_column_id: String, new_position: f64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE cards SET column_id = ?, position = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(new_column_id).bind(new_position).bind(card_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_card(&self, card_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM cards WHERE id = ?")
            .bind(card_id).execute(&self.pool).await?;
        Ok(())
    }
}
