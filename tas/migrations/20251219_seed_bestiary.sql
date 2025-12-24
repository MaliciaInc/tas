INSERT INTO bestiary_entries (id, universe_id, name, kind, habitat, description, danger, created_at, updated_at)
VALUES
    (
        lower(hex(randomblob(16))),
        (SELECT id FROM universes LIMIT 1),
    'Micófago Aullador',
    'Necrófago / Infectado',
    'Bosques densos, cuevas húmedas',
    'Lobo o bestia similar, mutado grotescamente por hongos parásitos que controlan su sistema nervioso. Su aullido es un gorgoteo húmedo que paraliza a las presas.',
    'High',
    datetime('now'), datetime('now')
    ),
(
    lower(hex(randomblob(16))),
    (SELECT id FROM universes LIMIT 1),
    'Dama del Cieno',
    'Anfibio / Maldito',
    'Pantanos, deltas de ríos y alcantarillas antiguas',
    'Parece una mujer anciana hecha de lodo y algas podridas. Se camufla en el barro y arrastra a los viajeros al fondo para ahogarlos. Sus garras segregan una toxina necrótica.',
    'Medium',
    datetime('now'), datetime('now')
),
(
    lower(hex(randomblob(16))),
    (SELECT id FROM universes LIMIT 1),
    'Caminante de Quitina',
    'Insectoide',
    'Cañones rocosos y nidos subterráneos',
    'Un depredador blindado de seis patas con mandíbulas capaces de partir acero. Son territoriales y vibran sus caparazones antes de atacar, produciendo un zumbido ensordecedor.',
    'High',
    datetime('now'), datetime('now')
),
(
    lower(hex(randomblob(16))),
    (SELECT id FROM universes LIMIT 1),
    'Espectro Cenizo',
    'Espectro',
    'Campos de batalla antiguos, ruinas incendiadas',
    'Espíritus de granjeros o soldados que murieron por fuego. Aparecen al mediodía entre nubes de ceniza y calor sofocante. Drenan la hidratación de sus víctimas hasta momificarlas.',
    'Extreme',
    datetime('now'), datetime('now')
),
(
    lower(hex(randomblob(16))),
    (SELECT id FROM universes LIMIT 1),
    'Centinela de Raíz',
    'Relicto / Elemental',
    'Corazón de bosques vírgenes',
    'Un tótem viviente de madera, hueso y astas de venado. No habla, pero comanda a los cuervos y lobos. Solo ataca si se daña el bosque, usando raíces que brotan explosivamente del suelo.',
    'Extreme',
    datetime('now'), datetime('now')
);