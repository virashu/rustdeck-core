use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct DeckConfig {
    pub cols: u32,
    pub rows: u32,
}
