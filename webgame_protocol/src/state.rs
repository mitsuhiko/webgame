use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateSnapshot {
    pub players: Vec<PlayerInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub game_id: Uuid,
    pub join_code: String,
}
