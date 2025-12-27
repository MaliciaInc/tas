-- Eliminamos el NOT NULL de universe_id
CREATE TABLE IF NOT EXISTS stories (
                                       id TEXT PRIMARY KEY,
                                       universe_id TEXT, -- <--- CAMBIO: Ya no es NOT NULL
                                       title TEXT NOT NULL,
                                       synopsis TEXT DEFAULT '',
                                       status TEXT DEFAULT 'draft',
                                       updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                       FOREIGN KEY(universe_id) REFERENCES universes(id) ON DELETE SET NULL
    );

-- (La tabla scenes queda igual)
CREATE TABLE IF NOT EXISTS scenes (
                                      id TEXT PRIMARY KEY,
                                      story_id TEXT NOT NULL,
                                      title TEXT NOT NULL,
                                      body TEXT DEFAULT '',
                                      position INTEGER DEFAULT 0,
                                      status TEXT DEFAULT 'draft',
                                      word_count INTEGER DEFAULT 0,
                                      updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                      FOREIGN KEY(story_id) REFERENCES stories(id) ON DELETE CASCADE
    );

CREATE INDEX idx_stories_universe ON stories(universe_id);
CREATE INDEX idx_scenes_story ON scenes(story_id);