use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::path::PathBuf;
use std::time::Duration;
use uuid::Uuid;
use flate2::write::GzEncoder;
use flate2::read::GzDecoder;
use flate2::Compression;
use std::io::{Read, Write};
use base64::{Engine as _, engine::general_purpose};

use crate::model::{
    Board, BoardColumn, Card, Creature, KanbanBoardData, Location, TimelineEvent, TimelineEra, Universe,
    UniverseSnapshot, UniverseSnapshotPayload, Story, Scene
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

        // 1. Apply migrations (schema v1 -> v5)
        db_migrations::apply(&pool).await?;

        // [TAS] Sync ProjectRoot identity (db_meta.project_kind) with the file extension.
        // Minimal, idempotent: does not change any data besides db_meta.
        let path_s = db_path.to_string_lossy();
        let project_kind = if path_s.ends_with(".novel") {
            "novel"
        } else if path_s.ends_with(".pmboard") {
            "pmboard"
        } else {
            "universe"
        };
        sqlx::query("UPDATE db_meta SET project_kind = ?")
            .bind(project_kind)
            .execute(&pool)
            .await?;
        crate::logger::info(&format!("DB ProjectKind synced: {}", project_kind));

        let db = Self { pool };

        // 2. System data repair (u-standalone, default boards)
        db.repair_integrity().await?;

        Ok(db)
    }

    pub async fn get_schema_version(&self) -> Result<i64, sqlx::Error> {
        db_migrations::read_schema_version(&self.pool).await
    }

    async fn repair_integrity(&self) -> Result<(), sqlx::Error> {
        // Asegurar que el universo "Standalone" existe (Invisible en UI principal)
        sqlx::query(
            "INSERT OR IGNORE INTO universes (id, name, description) VALUES \
             ('u-standalone', 'Standalone Stories', 'Historias independientes sin universo asociado.')",
        )
            .execute(&self.pool)
            .await?;

        // Asegurar Universo Arhelis (Demo)
        sqlx::query(
            "INSERT OR IGNORE INTO universes (id, name, description) VALUES \
             ('u-arhelis-01', 'Arhelis', 'Un mundo fracturado por la magia antigua.')",
        )
            .execute(&self.pool)
            .await?;

        // Asegurar Tablero Kanban
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

        self.repair_legacy_kanban().await?;

        Ok(())
    }

    async fn repair_legacy_kanban(&self) -> Result<(), sqlx::Error> {
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

        Ok(())
    }

    pub async fn inject_demo_data(&self, universe_id: String) -> Result<(), sqlx::Error> {
        if universe_id == "u-arhelis-01" {
            crate::db_seed::run_all(&self.pool, &universe_id).await?;
        }
        Ok(())
    }

    pub async fn reset_demo_data_scoped(&self, universe_id: String, scope: DemoResetScope) -> Result<(), sqlx::Error> {
        if universe_id != "u-arhelis-01" { return Ok(()); }
        let mut tx = self.pool.begin().await?;
        match scope {
            DemoResetScope::All => {
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')").execute(&mut *tx).await?;
                sqlx::query("DELETE FROM stories WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::Timeline => {
                sqlx::query("DELETE FROM timeline_events WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM timeline_eras WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::Locations => {
                sqlx::query("UPDATE bestiary_entries SET home_location_id = NULL WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
                sqlx::query("DELETE FROM locations WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::Bestiary => {
                sqlx::query("DELETE FROM bestiary_entries WHERE universe_id = ?").bind(&universe_id).execute(&mut *tx).await?;
            }
            DemoResetScope::PmTools => {
                sqlx::query("DELETE FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main')").execute(&mut *tx).await?;
            }
        }
        tx.commit().await?;
        self.repair_integrity().await?; // Re-ensure system data
        match scope {
            DemoResetScope::All => { crate::db_seed::run_all(&self.pool, &universe_id).await?; }
            DemoResetScope::Timeline => { crate::db_seed::timeline::seed(&self.pool, &universe_id).await?; }
            DemoResetScope::Locations => { crate::db_seed::locations::seed(&self.pool, &universe_id).await?; }
            DemoResetScope::Bestiary => { crate::db_seed::bestiary::seed(&self.pool, &universe_id).await?; }
            DemoResetScope::PmTools => { crate::db_seed::pm_tools::seed(&self.pool).await?; }
        }
        Ok(())
    }

    pub async fn validate_universe(&self, universe_id: String) -> Result<Vec<String>, sqlx::Error> {
        let mut issues: Vec<String> = Vec::new();
        let rows = sqlx::query("SELECT b.id, b.name, b.home_location_id FROM bestiary_entries b WHERE b.universe_id = ? AND b.home_location_id IS NOT NULL AND NOT EXISTS (SELECT 1 FROM locations l WHERE l.id = b.home_location_id)").bind(&universe_id).fetch_all(&self.pool).await?;
        for r in rows { issues.push(format!("Creature '{}' ({}) references missing location_id={}", r.get::<String, _>(1), r.get::<String, _>(0), r.get::<String, _>(2))); }
        Ok(issues)
    }

    pub async fn snapshot_list(&self, universe_id: String) -> Result<Vec<UniverseSnapshot>, sqlx::Error> {
        sqlx::query_as::<_, UniverseSnapshot>(
            "SELECT id, universe_id, name, datetime(created_at) as created_at FROM universe_snapshots WHERE universe_id = ? ORDER BY created_at DESC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn snapshot_create(&self, universe_id: String, name: String) -> Result<(), sqlx::Error> {
        let universe = sqlx::query_as::<_, Universe>("SELECT id, name, description, archived FROM universes WHERE id = ?").bind(&universe_id).fetch_one(&self.pool).await?;
        let creatures = self.get_creatures(universe_id.clone()).await?;
        let locations = self.get_locations_flat(universe_id.clone()).await?;
        let eras = self.get_timeline_eras(universe_id.clone()).await?;
        let events = self.get_timeline_events(universe_id.clone()).await?;
        let pm_cards: Vec<Card> = sqlx::query_as::<_, Card>("SELECT id, column_id, title, description, position, priority FROM cards WHERE column_id IN (SELECT id FROM board_columns WHERE board_id='board-main') ORDER BY position ASC").fetch_all(&self.pool).await?;
        let payload = UniverseSnapshotPayload { universe, creatures, locations, timeline_eras: eras, timeline_events: events, pm_cards };
        let json = serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string());
        let mut e = GzEncoder::new(Vec::new(), Compression::default());
        e.write_all(json.as_bytes()).map_err(|_| sqlx::Error::Protocol("Compress fail".into()))?;
        let compressed = e.finish().map_err(|_| sqlx::Error::Protocol("Compress fail".into()))?;
        let final_payload = general_purpose::STANDARD.encode(compressed);
        let sid = format!("snap-{}", Uuid::new_v4());

        // ZIP canonical column: payload_json
        sqlx::query("INSERT INTO universe_snapshots (id, universe_id, name, payload_json) VALUES (?, ?, ?, ?)")
            .bind(sid)
            .bind(universe_id)
            .bind(name)
            .bind(final_payload)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn snapshot_delete(&self, snapshot_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM universe_snapshots WHERE id = ?").bind(snapshot_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn snapshot_restore(&self, _snapshot_id: String) -> Result<(), sqlx::Error> { Ok(()) }

    // --- STANDARD CRUD ---
    // [FILTRO] u-standalone se oculta aquí
    pub async fn get_all_universes(&self) -> Result<Vec<Universe>, sqlx::Error> {
        sqlx::query_as::<_, Universe>(
            "SELECT id, name, description, archived FROM universes WHERE id != 'u-standalone' ORDER BY created_at DESC"
        )
            .fetch_all(&self.pool)
            .await
    }

    pub async fn create_universe(&self, name: String, description: String) -> Result<(), sqlx::Error> {
        let uid = format!("u-{}", Uuid::new_v4());
        sqlx::query("INSERT INTO universes (id, name, description) VALUES (?, ?, ?)")
            .bind(uid)
            .bind(name)
            .bind(description)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_universe(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("PRAGMA foreign_keys = ON").execute(&self.pool).await?;
        sqlx::query("DELETE FROM universes WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_creatures(&self, universe_id: String) -> Result<Vec<Creature>, sqlx::Error> {
        sqlx::query_as::<_, Creature>(
            "SELECT id, name, kind, habitat, description, danger, home_location_id, archived FROM bestiary_entries WHERE universe_id = ? ORDER BY archived ASC, created_at DESC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_creature(&self, c: Creature, universe_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO bestiary_entries (id, universe_id, name, kind, habitat, description, danger, home_location_id, archived, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET name=excluded.name, kind=excluded.kind, habitat=excluded.habitat, description=excluded.description, danger=excluded.danger, home_location_id=excluded.home_location_id, archived=excluded.archived, updated_at=datetime('now')")
            .bind(c.id).bind(universe_id).bind(c.name).bind(c.kind).bind(c.habitat).bind(c.description).bind(c.danger).bind(c.home_location_id).bind(c.archived)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn set_creature_archived(&self, id: String, archived: bool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE bestiary_entries SET archived = ? WHERE id = ?").bind(archived).bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_creature(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM bestiary_entries WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_locations_flat(&self, universe_id: String) -> Result<Vec<Location>, sqlx::Error> {
        sqlx::query_as::<_, Location>(
            "SELECT id, universe_id, parent_id, name, description, kind FROM locations WHERE universe_id = ? ORDER BY name ASC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_location(&self, l: Location) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO locations (id, universe_id, parent_id, name, description, kind, updated_at) VALUES (?, ?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET parent_id=excluded.parent_id, name=excluded.name, description=excluded.description, kind=excluded.kind, updated_at=datetime('now')")
            .bind(l.id).bind(l.universe_id).bind(l.parent_id).bind(l.name).bind(l.description).bind(l.kind)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_location(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM locations WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_timeline_events(&self, universe_id: String) -> Result<Vec<TimelineEvent>, sqlx::Error> {
        sqlx::query_as::<_, TimelineEvent>(
            "SELECT id, universe_id, title, description, year, COALESCE(display_date, '') AS display_date, importance, kind, color, location_id FROM timeline_events WHERE universe_id = ? ORDER BY year ASC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_timeline_eras(&self, universe_id: String) -> Result<Vec<TimelineEra>, sqlx::Error> {
        sqlx::query_as::<_, TimelineEra>(
            "SELECT id, universe_id, name, start_year, end_year, description, color FROM timeline_eras WHERE universe_id = ? ORDER BY start_year ASC"
        )
            .bind(universe_id)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn upsert_timeline_event(&self, e: TimelineEvent) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO timeline_events (id, universe_id, title, description, year, display_date, importance, kind, color, location_id, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET title=excluded.title, description=excluded.description, year=excluded.year, display_date=excluded.display_date, importance=excluded.importance, kind=excluded.kind, color=excluded.color, location_id=excluded.location_id, updated_at=datetime('now')")
            .bind(e.id).bind(e.universe_id).bind(e.title).bind(e.description).bind(e.year).bind(e.display_date).bind(e.importance).bind(e.kind).bind(e.color).bind(e.location_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_timeline_event(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM timeline_events WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn upsert_timeline_era(&self, e: TimelineEra) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO timeline_eras (id, universe_id, name, start_year, end_year, description, color, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET name=excluded.name, start_year=excluded.start_year, end_year=excluded.end_year, description=excluded.description, color=excluded.color")
            .bind(e.id).bind(e.universe_id).bind(e.name).bind(e.start_year).bind(e.end_year).bind(e.description).bind(e.color)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_timeline_era(&self, id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM timeline_eras WHERE id = ?").bind(id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_all_boards(&self) -> Result<Vec<Board>, sqlx::Error> {
        sqlx::query_as::<_, Board>("SELECT id, name, kind FROM boards ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_kanban_data(&self, board_id: String) -> Result<KanbanBoardData, sqlx::Error> {
        let board = sqlx::query_as::<_, Board>("SELECT id, name, kind FROM boards WHERE id = ?").bind(&board_id).fetch_one(&self.pool).await?;
        let columns_raw = sqlx::query_as::<_, BoardColumn>("SELECT id, board_id, name, position FROM board_columns WHERE board_id = ? ORDER BY position ASC").bind(&board_id).fetch_all(&self.pool).await?;
        let mut columns_data = Vec::new();
        for col in columns_raw {
            let cards = sqlx::query_as::<_, Card>("SELECT id, column_id, title, description, position, priority FROM cards WHERE column_id = ? ORDER BY position ASC").bind(&col.id).fetch_all(&self.pool).await?;
            columns_data.push((col, cards));
        }
        Ok(KanbanBoardData { board, columns: columns_data })
    }

    pub async fn upsert_card(&self, c: Card) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO cards (id, column_id, title, description, position, priority, updated_at) VALUES (?, ?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET column_id=excluded.column_id, title=excluded.title, description=excluded.description, position=excluded.position, priority=excluded.priority, updated_at=datetime('now')")
            .bind(c.id).bind(c.column_id).bind(c.title).bind(c.description).bind(c.position).bind(c.priority)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn move_card(&self, card_id: String, new_column_id: String, new_position: f64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE cards SET column_id = ?, position = ?, updated_at = datetime('now') WHERE id = ?")
            .bind(new_column_id).bind(new_position).bind(card_id)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_card(&self, card_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM cards WHERE id = ?").bind(card_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn create_board(&self, name: String) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let bid = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO boards (id, name, kind) VALUES (?, ?, 'kanban')").bind(&bid).bind(name).execute(&mut *tx).await?;
        let cols = vec![("On-Hold", 0), ("To Do", 1), ("In Progress", 2), ("Done", 3)];
        for (cname, pos) in cols {
            let cid = Uuid::new_v4().to_string();
            sqlx::query("INSERT INTO board_columns (id, board_id, name, position) VALUES (?, ?, ?, ?)").bind(cid).bind(&bid).bind(cname).bind(pos).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    pub async fn delete_board(&self, board_id: String) -> Result<(), sqlx::Error> {
        let mut tx = self.pool.begin().await?;
        let cols: Vec<(String,)> = sqlx::query_as("SELECT id FROM board_columns WHERE board_id = ?").bind(&board_id).fetch_all(&mut *tx).await?;
        for (cid,) in cols {
            sqlx::query("DELETE FROM cards WHERE column_id = ?").bind(&cid).execute(&mut *tx).await?;
        }
        sqlx::query("DELETE FROM board_columns WHERE board_id = ?").bind(&board_id).execute(&mut *tx).await?;
        sqlx::query("DELETE FROM boards WHERE id = ?").bind(&board_id).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn rebalance_column(&self, column_id: String) -> Result<(), sqlx::Error> {
        let cards: Vec<(String,)> = sqlx::query_as("SELECT id FROM cards WHERE column_id = ? ORDER BY position ASC").bind(&column_id).fetch_all(&self.pool).await?;
        let mut tx = self.pool.begin().await?;
        for (i, (id,)) in cards.into_iter().enumerate() {
            let new_pos = (i as f64 + 1.0) * 1000.0;
            sqlx::query("UPDATE cards SET position = ? WHERE id = ?").bind(new_pos).bind(id).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(())
    }

    // --- THE FORGE IMPL ---
    pub async fn get_stories(&self, universe_id: String) -> Result<Vec<Story>, sqlx::Error> {
        sqlx::query_as::<_, Story>("SELECT id, universe_id, title, synopsis, status FROM stories WHERE universe_id = ? ORDER BY updated_at DESC").bind(universe_id).fetch_all(&self.pool).await
    }

    pub async fn create_story(&self, universe_id: String, title: String) -> Result<(), sqlx::Error> {
        let sid = format!("story-{}", Uuid::new_v4());
        sqlx::query("INSERT INTO stories (id, universe_id, title) VALUES (?, ?, ?)").bind(sid).bind(universe_id).bind(title).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn upsert_story(&self, s: Story) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE stories SET title = ?, synopsis = ?, status = ?, updated_at = datetime('now') WHERE id = ?").bind(s.title).bind(s.synopsis).bind(s.status).bind(s.id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_story(&self, story_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM stories WHERE id = ?").bind(story_id).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn get_scenes(&self, story_id: String) -> Result<Vec<Scene>, sqlx::Error> {
        sqlx::query_as::<_, Scene>("SELECT id, story_id, title, body, position, status, word_count FROM scenes WHERE story_id = ? ORDER BY position ASC").bind(story_id).fetch_all(&self.pool).await
    }

    pub async fn create_scene(&self, story_id: String, title: String) -> Result<(), sqlx::Error> {
        let sid = format!("scene-{}", Uuid::new_v4());
        let (max_pos,): (Option<i64>,) = sqlx::query_as("SELECT MAX(position) FROM scenes WHERE story_id = ?").bind(&story_id).fetch_one(&self.pool).await?;
        let pos = max_pos.unwrap_or(0) + 1;
        sqlx::query("INSERT INTO scenes (id, story_id, title, position) VALUES (?, ?, ?, ?)").bind(sid).bind(story_id).bind(title).bind(pos).execute(&self.pool).await?;
        Ok(())
    }

    pub async fn upsert_scene(&self, s: Scene) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT INTO scenes (id, story_id, title, body, position, status, word_count, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, datetime('now')) ON CONFLICT(id) DO UPDATE SET title=excluded.title, body=excluded.body, position=excluded.position, status=excluded.status, word_count=excluded.word_count, updated_at=datetime('now')")
            .bind(s.id).bind(s.story_id).bind(s.title).bind(s.body).bind(s.position).bind(s.status).bind(s.word_count)
            .execute(&self.pool).await?;
        Ok(())
    }

    pub async fn delete_scene(&self, scene_id: String) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM scenes WHERE id = ?").bind(scene_id).execute(&self.pool).await?;
        Ok(())
    }
}

// --- Internal schema migrations (inlined from former src/db_migrations.rs) ---
mod db_migrations {
    use sqlx::{Row, SqlitePool};

    // VERSIÓN 5: Introduce db_meta (ProjectRoot identity)
    const CURRENT_SCHEMA_VERSION: i64 = 5;

    /// Apply schema migrations in a forward-only, idempotent way.
    pub async fn apply(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS app_meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    "#,
        )
            .execute(pool)
            .await?;

        let current = read_schema_version(pool).await?;

        if current < CURRENT_SCHEMA_VERSION {
            for v in (current + 1)..=CURRENT_SCHEMA_VERSION {
                apply_migration(pool, v).await?;
                set_schema_version(pool, v).await?;
            }
        }

        // Backward-compat repair: idempotent
        repair_columns(pool).await?;

        Ok(())
    }

    pub async fn read_schema_version(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        get_schema_version(pool).await
    }

    async fn apply_migration(pool: &SqlitePool, version: i64) -> Result<(), sqlx::Error> {
        match version {
            1 => migration_v0001(pool).await,
            2 => migration_v0002(pool).await,
            3 => migration_v0003(pool).await,
            4 => migration_v0004(pool).await,
            5 => migration_v0005(pool).await,
            _ => Ok(()),
        }
    }

    async fn get_schema_version(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
        let row = sqlx::query("SELECT value FROM app_meta WHERE key = 'schema_version'")
            .fetch_optional(pool)
            .await?;

        if let Some(r) = row {
            let v: String = r.get("value");
            Ok(v.parse::<i64>().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    async fn set_schema_version(pool: &SqlitePool, version: i64) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO app_meta (key, value) VALUES ('schema_version', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
            .bind(version.to_string())
            .execute(pool)
            .await?;
        Ok(())
    }

    async fn repair_columns(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        add_column_if_missing(pool, "universes", "archived", "INTEGER NOT NULL DEFAULT 0").await?;
        add_column_if_missing(pool, "locations", "kind", "TEXT NOT NULL DEFAULT 'Place'").await?;
        add_column_if_missing(pool, "timeline_events", "location_id", "TEXT").await?;
        add_column_if_missing(pool, "boards", "kind", "TEXT NOT NULL DEFAULT 'kanban'").await?;

        // snapshots v2 (ZIP canonical): universe_snapshots.payload_json (base64 gz)
        add_column_if_missing(pool, "universe_snapshots", "payload_json", "TEXT NOT NULL DEFAULT ''").await?;
        add_column_if_missing(pool, "universe_snapshots", "created_at", "DATETIME DEFAULT CURRENT_TIMESTAMP").await?;

        Ok(())
    }

    async fn migration_v0001(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS universes (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            archived INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS locations (
            id TEXT PRIMARY KEY,
            universe_id TEXT NOT NULL,
            parent_id TEXT,
            name TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            kind TEXT NOT NULL DEFAULT 'Place',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE,
            FOREIGN KEY(parent_id) REFERENCES locations(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS bestiary_entries (
            id TEXT PRIMARY KEY,
            universe_id TEXT NOT NULL,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT '',
            habitat TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT '',
            danger TEXT NOT NULL DEFAULT '',
            home_location_id TEXT,
            archived INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE,
            FOREIGN KEY(home_location_id) REFERENCES locations(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS timeline_eras (
            id TEXT PRIMARY KEY,
            universe_id TEXT NOT NULL,
            name TEXT NOT NULL,
            start_year INTEGER NOT NULL DEFAULT 0,
            end_year INTEGER NOT NULL DEFAULT 0,
            description TEXT NOT NULL DEFAULT '',
            color TEXT NOT NULL DEFAULT '#ffffff',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS timeline_events (
            id TEXT PRIMARY KEY,
            universe_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            year INTEGER NOT NULL DEFAULT 0,
            display_date TEXT NOT NULL DEFAULT '',
            importance TEXT NOT NULL DEFAULT 'Normal',
            kind TEXT NOT NULL DEFAULT 'Event',
            color TEXT NOT NULL DEFAULT '#ffffff',
            location_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE
        );
    "#,
        )
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn migration_v0002(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS boards (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'kanban',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS board_columns (
            id TEXT PRIMARY KEY,
            board_id TEXT NOT NULL,
            name TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(board_id) REFERENCES boards(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            column_id TEXT NOT NULL,
            title TEXT NOT NULL,
            description TEXT NOT NULL DEFAULT '',
            position REAL NOT NULL DEFAULT 0,
            priority TEXT NOT NULL DEFAULT 'Normal',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(column_id) REFERENCES board_columns(id) ON DELETE CASCADE
        );
    "#,
        )
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn migration_v0003(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS stories (
            id TEXT PRIMARY KEY,
            universe_id TEXT NOT NULL,
            title TEXT NOT NULL,
            synopsis TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'Draft',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS scenes (
            id TEXT PRIMARY KEY,
            story_id TEXT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            position INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'Draft',
            word_count INTEGER NOT NULL DEFAULT 0,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(story_id) REFERENCES stories(id) ON DELETE CASCADE
        );
    "#,
        )
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn migration_v0004(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS universe_snapshots (
            id TEXT PRIMARY KEY,
            universe_id TEXT NOT NULL,
            name TEXT NOT NULL,
            payload_json TEXT NOT NULL DEFAULT '',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE
        );
    "#,
        )
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn migration_v0005(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS db_meta (
            schema_version INTEGER NOT NULL DEFAULT 0,
            project_kind TEXT NOT NULL DEFAULT 'universe',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            app_version TEXT NOT NULL DEFAULT ''
        );

        -- Si ya existe un registro, no lo duplicamos.
        INSERT INTO db_meta (schema_version, project_kind, app_version)
        SELECT 5, 'universe', ''
        WHERE NOT EXISTS (SELECT 1 FROM db_meta);
        "#,
        )
            .execute(pool)
            .await?;

        // Mantener schema_version en sync con el estado actual (idempotente)
        sqlx::query("UPDATE db_meta SET schema_version = 5")
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn add_column_if_missing(pool: &SqlitePool, table: &str, col: &str, ddl: &str) -> Result<(), sqlx::Error> {
        let q = format!(
            "SELECT COUNT(*) as cnt FROM pragma_table_info('{}') WHERE name='{}'",
            table.replace('\'', "''"),
            col.replace('\'', "''"),
        );
        let row = sqlx::query(&q).fetch_one(pool).await?;
        let cnt: i64 = row.try_get("cnt")?;
        if cnt == 0 {
            let alter = format!("ALTER TABLE {} ADD COLUMN {} {}", table, col, ddl);
            sqlx::query(&alter).execute(pool).await?;
        }
        Ok(())
    }
}
