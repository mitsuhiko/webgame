use serde::{Serialize, Deserialize};

use crate::player::Player;

#[derive(Serialize, Deserialize)]
pub struct GameStateSnapshot {
    pub players: Vec<Player>,
}
