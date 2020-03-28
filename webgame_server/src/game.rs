use std::collections::BTreeMap;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use uuid::Uuid;

use crate::board::Board;
use crate::protocol::{
    GameInfo, GameStateSnapshot, Message, PlayerDisconnectedMessage, PlayerRole, Team,
};
use crate::universe::Universe;

enum GameProgression {
    Lobby,
    Pregame,
}

pub struct GamePlayerState {
    player_id: Uuid,
    role: PlayerRole,
    team: Option<Team>,
    ready: bool,
}

pub struct GameState {
    players: BTreeMap<Uuid, GamePlayerState>,
    progression: GameProgression,
    board: Board,
}

pub struct Game {
    id: Uuid,
    join_code: String,
    universe: Weak<Universe>,
    game_state: Arc<Mutex<GameState>>,
}

impl Game {
    pub fn new(join_code: String, universe: Arc<Universe>) -> Game {
        Game {
            id: Uuid::new_v4(),
            join_code,
            universe: Arc::downgrade(&universe),
            game_state: Arc::new(Mutex::new(GameState {
                players: BTreeMap::new(),
                progression: GameProgression::Lobby,
                board: Board::new(),
            })),
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn join_code(&self) -> &str {
        &self.join_code
    }

    pub fn game_info(&self) -> GameInfo {
        GameInfo {
            game_id: self.id,
            join_code: self.join_code.to_string(),
        }
    }

    pub async fn snapshot_for(&self, player_id: Uuid) -> GameStateSnapshot {
        let state = self.game_state.lock().await;
        let universe = self.universe();
        let mut players = vec![];
        let mut role = PlayerRole::Spectator;
        for (&other_player_id, state) in state.players.iter() {
            if player_id == other_player_id {
                role = state.role;
            }
            if let Some(player_info) = universe.get_player_info(other_player_id).await {
                players.push(player_info);
            }
        }
        GameStateSnapshot {
            players,
            tiles: state.board.tiles_for_role(role),
        }
    }

    pub async fn is_joinable(&self) -> bool {
        matches!(
            self.game_state.lock().await.progression,
            GameProgression::Lobby
        )
    }

    pub fn universe(&self) -> Arc<Universe> {
        self.universe.upgrade().unwrap()
    }

    pub async fn add_player(&self, player_id: Uuid) {
        let universe = self.universe();
        if !universe
            .set_player_game_id(player_id, Some(self.id()))
            .await
        {
            return;
        }

        let mut game_state = self.game_state.lock().await;
        if game_state.players.contains_key(&player_id) {
            return;
        }
        game_state.players.insert(
            player_id,
            GamePlayerState {
                player_id,
                role: PlayerRole::Spectator,
                team: None,
                ready: false,
            },
        );

        if let Some(player_info) = universe.get_player_info(player_id).await {
            drop(game_state);
            self.broadcast(&Message::PlayerConnected(player_info)).await;
        }
    }

    pub async fn remove_player(&self, player_id: Uuid) {
        self.universe().set_player_game_id(player_id, None).await;

        let mut game_state = self.game_state.lock().await;
        if game_state.players.remove(&player_id).is_some() {
            drop(game_state);
            self.broadcast(&Message::PlayerDisconnected(PlayerDisconnectedMessage {
                player_id,
            }))
            .await;
        }

        if self.is_empty().await {
            self.universe().remove_game(self.id()).await;
        }
    }

    pub async fn mark_player_ready(&self, player_id: Uuid) {
        let mut game_state = self.game_state.lock().await;
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.ready = true;
        }
        if game_state.players.values().all(|x| x.ready) {
            game_state.progression = GameProgression::Pregame;
            drop(game_state);
            self.broadcast(&Message::PregameStarted).await;
        }
    }

    pub async fn broadcast(&self, message: &Message) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_state in game_state.players.values() {
            universe.send(player_state.player_id, message).await;
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.game_state.lock().await.players.is_empty()
    }
}
