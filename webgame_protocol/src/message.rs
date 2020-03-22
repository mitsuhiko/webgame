use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::state::GameStateSnapshot;

#[derive(Serialize, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Hello(HelloRequest),
    RefreshGameState,
    SendText(SendTextRequest),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Ok(SuccessResponse),
    Error(ProtocolError),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SuccessResponse {
    /// Simple acknowledgement without response.
    Ack,
    /// Response to hello.
    Authenticated(AuthenticatedResponse),
    /// The current game snapshot.
    GameStateSnapshot(GameStateSnapshot),
}

#[derive(Serialize, Deserialize)]
pub struct AuthenticatedResponse {
    pub player_id: Uuid,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "packet", rename_all = "snake_case")]
pub enum Packet {
    Response(Response),
    Message(Message),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorKind {
    /// Client tried to authenticate twice
    AlreadyAuthenticated,
    /// Client sent in some garbage
    InvalidRequest,
    /// This should never happen.
    InternalError,
    /// Nickname is already in use.
    NicknameInUse,
}

#[derive(Serialize, Deserialize)]
pub struct ProtocolError {
    kind: ProtocolErrorKind,
    message: Option<String>,
}

impl ProtocolError {
    pub fn new<S: Into<String>>(kind: ProtocolErrorKind, s: S) -> ProtocolError {
        ProtocolError {
            kind,
            message: Some(s.into()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct HelloRequest {
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub struct SendTextRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Chat(ChatMessage),
    PlayerConnected(PlayerConnectedMessage),
    PlayerDisconnected(PlayerDisconnectedMessage),
}

#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub player_id: Uuid,
    pub text: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerConnectedMessage {
    pub player_id: Uuid,
    pub nickname: String,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerDisconnectedMessage {
    pub player_id: Uuid,
}