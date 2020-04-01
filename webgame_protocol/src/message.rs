use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::game::{GameInfo, GamePlayerState, GameStateSnapshot, PlayerRole, Team};
use crate::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    Authenticate(AuthenticateCommand),
    SendText(SendTextCommand),
    NewGame,
    JoinGame(JoinGameCommand),
    LeaveGame,
    SetPlayerRole(SetPlayerRoleCommand),
    RequestGameStateSnapshot,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorKind {
    /// Client tried to authenticate twice
    AlreadyAuthenticated,
    /// Tried to do something while unauthenticated
    NotAuthenticated,
    /// Client sent in some garbage
    InvalidCommand,
    /// Cannot be done at this time
    BadState,
    /// Something wasn't found
    NotFound,
    /// Invalid input.
    BadInput,
    /// This should never happen.
    InternalError,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProtocolError {
    kind: ProtocolErrorKind,
    message: String,
}

impl ProtocolError {
    pub fn new<S: Into<String>>(kind: ProtocolErrorKind, s: S) -> ProtocolError {
        ProtocolError {
            kind,
            message: s.into(),
        }
    }

    pub fn kind(&self) -> ProtocolErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthenticateCommand {
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendTextCommand {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JoinGameCommand {
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetPlayerRoleCommand {
    pub team: Option<Team>,
    pub role: PlayerRole,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Chat(ChatMessage),
    PlayerConnected(GamePlayerState),
    PlayerDisconnected(PlayerDisconnectedMessage),
    PregameStarted,
    GameJoined(GameInfo),
    GameLeft,
    Authenticated(PlayerInfo),
    Error(ProtocolError),
    GameStateSnapshot(GameStateSnapshot),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    pub player_id: Uuid,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerDisconnectedMessage {
    pub player_id: Uuid,
}
