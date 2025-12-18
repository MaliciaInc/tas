#[derive(Debug, Clone)]
pub struct Universe {
    pub id: String,
    pub name: String,
    pub description: String,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct Creature {
    pub name: String,
    pub kind: String,
    pub habitat: String,
    pub description: String,
    pub danger: String,
}
