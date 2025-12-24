CREATE TABLE IF NOT EXISTS locations (
                                         id TEXT PRIMARY KEY,
                                         universe_id TEXT NOT NULL,
                                         parent_id TEXT, -- Autoreferencia: NULL si es raíz, ID si es hijo
                                         name TEXT NOT NULL,
                                         description TEXT DEFAULT '',
                                         kind TEXT NOT NULL DEFAULT 'Place', -- 'Region', 'City', 'Landmark'
                                         created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                         updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,

                                         FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE CASCADE,
    FOREIGN KEY(parent_id) REFERENCES locations(id) ON DELETE CASCADE
    );