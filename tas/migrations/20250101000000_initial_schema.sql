-- 1. CREACIÓN DE TABLAS
CREATE TABLE IF NOT EXISTS universes (
                                         id TEXT PRIMARY KEY,
                                         name TEXT NOT NULL,
                                         description TEXT,
                                         archived BOOLEAN DEFAULT 0,
                                         created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                         updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS bestiary_entries (
                                                id TEXT PRIMARY KEY,
                                                universe_id TEXT NOT NULL,
                                                name TEXT NOT NULL,
                                                kind TEXT,
                                                habitat TEXT,
                                                description TEXT,
                                                danger TEXT,
                                                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                                FOREIGN KEY(universe_id) REFERENCES universes(id)
    );

CREATE TABLE IF NOT EXISTS boards (
                                      id TEXT PRIMARY KEY,
                                      name TEXT NOT NULL,
                                      kind TEXT NOT NULL, -- 'kanban', 'list'
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
                                     description TEXT,
                                     position INTEGER NOT NULL,
                                     created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                     updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                                     FOREIGN KEY(column_id) REFERENCES board_columns(id)
    );

-- 2. DATOS SEMILLA (CRUCIAL PARA QUE NO SE QUEDE CARGANDO)
-- Insertamos el Universo "Arhelis"
INSERT OR IGNORE INTO universes (id, name, description)
VALUES ('u-arhelis-01', 'Arhelis', 'Un mundo fracturado por la magia antigua y la tecnología olvidada.');

-- Insertamos el Tablero Principal (ID: board-main es lo que busca el código)
INSERT OR IGNORE INTO boards (id, name, kind)
VALUES ('board-main', 'Development Roadmap', 'kanban');

-- Insertamos Columnas para ese tablero
INSERT OR IGNORE INTO board_columns (id, board_id, name, position) VALUES
('col-todo', 'board-main', 'To Do', 0),
('col-progress', 'board-main', 'In Progress', 1),
('col-done', 'board-main', 'Done', 2);

-- Insertamos una tarjeta de ejemplo
INSERT OR IGNORE INTO cards (id, column_id, title, description, position)
VALUES ('card-01', 'col-todo', 'Bienvenido a TAS', 'Arrastra esta tarjeta para probar el Kanban.', 0);