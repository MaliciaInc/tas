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
