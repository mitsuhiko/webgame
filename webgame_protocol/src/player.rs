use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub id: Uuid,
    pub nickname: String,
}
