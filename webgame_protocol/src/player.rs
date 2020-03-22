use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInfo {
    pub id: Uuid,
    pub nickname: String,
}
