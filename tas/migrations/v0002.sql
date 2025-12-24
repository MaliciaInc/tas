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
