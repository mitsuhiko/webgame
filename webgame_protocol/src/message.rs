use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::player::PlayerInfo;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Request {
    Authenticate(AuthenticateRequest),
    SendText(SendTextRequest),
    NewGame,
    JoinGame(JoinGameRequest),
    MarkReady,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Response {
    Ok(SuccessResponse),
    Error(ProtocolError),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SuccessResponse {
    /// Simple acknowledgement without response.
    Ack,
    /// Response when a new game was created.
    NewGame(NewGameResponse),
    /// Response when a game was joined.
    GameJoined(GameJoinedResponse),
    /// Response to hello.
    Authenticated(AuthenticatedResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGameResponse {
    pub game_id: Uuid,
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GameJoinedResponse {
    pub game_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticatedResponse {
    pub player_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "packet", rename_all = "snake_case")]
pub enum Packet {
    Response(Response),
    Message(Message),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorKind {
    /// Client tried to authenticate twice
    AlreadyAuthenticated,
    /// Tried to do something while unauthenticated
    NotAuthenticated,
    /// Client sent in some garbage
    InvalidRequest,
    /// Something wasn't found
    NotFound,
    /// This should never happen.
    InternalError,
}

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthenticateRequest {
    pub nickname: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SendTextRequest {
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinGameRequest {
    pub join_code: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Chat(ChatMessage),
    PlayerConnected(PlayerConnectedMessage),
    PlayerDisconnected(PlayerDisconnectedMessage),
    PregameStarted,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    pub player_id: Uuid,
    pub text: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerConnectedMessage {
    pub player_info: PlayerInfo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerDisconnectedMessage {
    pub player_id: Uuid,
}
