use serde::{Deserialize, Serialize};

use crate::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug)]
pub struct GameStateSnapshot {
    pub players: Vec<PlayerInfo>,
}
