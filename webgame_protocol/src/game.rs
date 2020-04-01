use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum Turn {
    Pregame,
    Intermission,
    RedSpymasterThinking,
    BlueSpymasterThinking,
    RedOperativesGuessing,
    BlueOperativesGuessing,
    Endgame,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameStateSnapshot {
    pub players: Vec<GamePlayerState>,
    pub tiles: Vec<Tile>,
    pub turn: Turn,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameInfo {
    pub game_id: Uuid,
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Team {
    Red,
    Blue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tile {
    pub codeword: String,
    pub character: Character,
    pub spotted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlayerRole {
    Spymaster,
    Operative,
    Spectator,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Character {
    /// This is not a real character but it shows up for board projections
    /// of non spymaster players.
    Unknown,
    RedAgent,
    BlueAgent,
    Bystander,
    Assassin,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GamePlayerState {
    pub player: PlayerInfo,
    pub team: Option<Team>,
    pub role: PlayerRole,
    pub ready: bool,
}
