use sqlx::{Row, SqlitePool};

const CURRENT_SCHEMA_VERSION: i64 = 2;

// Migration SQL is embedded to avoid include_str! path issues on Windows
// and to keep the project self-contained.
const V0001_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS universes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    archived BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS boards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    kind TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS board_columns (
    id TEXT PRIMARY KEY,
    board_id TEXT NOT NULL,
    name TEXT NOT NULL,
    position INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(board_id) REFERENCES boards(id)
);

CREATE TABLE IF NOT EXISTS cards (
    id TEXT PRIMARY KEY,
    column_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    position REAL NOT NULL DEFAULT 0.0,
    priority TEXT NOT NULL DEFAULT 'Medium',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(column_id) REFERENCES board_columns(id)
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
    archived BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE
);
"#;

const V0002_SQL: &str = r#"
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

CREATE TABLE IF NOT EXISTS timeline_eras (
    id TEXT PRIMARY KEY,
    universe_id TEXT NOT NULL,
    name TEXT NOT NULL,
    start_year INTEGER NOT NULL,
    end_year INTEGER,
    description TEXT DEFAULT '',
    color TEXT DEFAULT '#6366F1',
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS timeline_events (
    id TEXT PRIMARY KEY,
    universe_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT DEFAULT '',
    year INTEGER NOT NULL,
    display_date TEXT,
    importance TEXT DEFAULT 'Normal',
    kind TEXT DEFAULT 'General',
    color TEXT DEFAULT '#A1A1AA',
    location_id TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE,
    FOREIGN KEY(location_id) REFERENCES locations(id) ON DELETE SET NULL
);
"#;

pub async fn apply(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Metadata table for schema versioning
    sqlx::query("CREATE TABLE IF NOT EXISTS app_meta (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        .execute(pool)
        .await?;

    let mut version = get_schema_version(pool).await?;

    // Apply migrations sequentially
    while version < CURRENT_SCHEMA_VERSION {
        let next = version + 1;
        apply_migration(pool, next).await?;
        set_schema_version(pool, next).await?;
        version = next;
    }

    // Future-proof: if DB is newer, do not explode
    if version > CURRENT_SCHEMA_VERSION {
        crate::logger::info(&format!(
            "DB schema_version={} is newer than app version={}. Continuing.",
            version, CURRENT_SCHEMA_VERSION
        ));
    }

    Ok(())
}

async fn apply_migration(pool: &SqlitePool, version: i64) -> Result<(), sqlx::Error> {
    match version {
        1 => migration_v0001(pool).await,
        2 => migration_v0002(pool).await,
        _ => Ok(()),
    }
}

async fn get_schema_version(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query("SELECT value FROM app_meta WHERE key='schema_version'")
        .fetch_optional(pool)
        .await?;

    Ok(match row {
        Some(r) => {
            let v: String = r.try_get(0)?;
            v.parse::<i64>().unwrap_or(0)
        }
        None => 0,
    })
}

async fn set_schema_version(pool: &SqlitePool, v: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO app_meta (key, value) VALUES ('schema_version', ?) \
         ON CONFLICT(key) DO UPDATE SET value=excluded.value",
    )
        .bind(v.to_string())
        .execute(pool)
        .await?;
    Ok(())
}

async fn migration_v0001(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create base schema (idempotent due to IF NOT EXISTS)
    sqlx::query(V0001_SQL).execute(pool).await?;
    Ok(())
}

async fn migration_v0002(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Add locations + timeline tables (idempotent due to IF NOT EXISTS)
    sqlx::query(V0002_SQL).execute(pool).await?;

    // Defensive column additions for existing DBs
    add_column_if_missing(pool, "bestiary_entries", "archived", "BOOLEAN NOT NULL DEFAULT 0").await?;
    add_column_if_missing(pool, "bestiary_entries", "home_location_id", "TEXT").await?;
    add_column_if_missing(pool, "cards", "priority", "TEXT NOT NULL DEFAULT 'Medium'").await?;

    Ok(())
}

async fn add_column_if_missing(
    pool: &SqlitePool,
    table: &str,
    col: &str,
    decl: &str,
) -> Result<(), sqlx::Error> {
    let q = format!(
        "SELECT COUNT(*) as cnt FROM pragma_table_info('{}') WHERE name='{}'",
        table, col
    );
    let row = sqlx::query(&q).fetch_one(pool).await?;
    let cnt: i64 = row.try_get(0)?;
    if cnt == 0 {
        let alter = format!("ALTER TABLE {} ADD COLUMN {} {}", table, col, decl);
        sqlx::query(&alter).execute(pool).await?;
    }
    Ok(())
}
