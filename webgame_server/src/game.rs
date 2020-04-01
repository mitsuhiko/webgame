use std::collections::BTreeMap;
use std::sync::{Arc, Weak};
use tokio::sync::Mutex;

use uuid::Uuid;

use crate::board::Board;
use crate::protocol::{
    GameInfo, GamePlayerState, GameStateSnapshot, Message, PlayerDisconnectedMessage, PlayerRole,
    Team, Turn,
};
use crate::universe::Universe;

pub struct GameState {
    players: BTreeMap<Uuid, GamePlayerState>,
    turn: Turn,
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
                turn: Turn::Pregame,
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
        let mut players = vec![];
        let mut reveal = false;
        for (&other_player_id, player_state) in state.players.iter() {
            if player_id == other_player_id
                && player_state.role == PlayerRole::Spymaster
                && state.turn != Turn::Pregame
            {
                reveal = true;
            }
            players.push(player_state.clone());
        }
        GameStateSnapshot {
            players,
            tiles: state.board.tiles(reveal),
            turn: state.turn,
        }
    }

    pub async fn is_joinable(&self) -> bool {
        self.game_state.lock().await.turn == Turn::Pregame
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

        // TODO: `set_player_game_id` also looks up.
        let player_info = match universe.get_player_info(player_id).await {
            Some(player_info) => player_info,
            None => return,
        };

        let state = GamePlayerState {
            player: player_info,
            role: PlayerRole::Spectator,
            team: None,
            ready: false,
        };
        game_state.players.insert(state.player.id, state.clone());

        drop(game_state);
        self.broadcast(&Message::PlayerConnected(state)).await;
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

    pub async fn set_player_role_and_team(
        &self,
        player_id: Uuid,
        role: PlayerRole,
        team: Option<Team>,
    ) {
        let mut game_state = self.game_state.lock().await;
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.role = role;
            player_state.team = team;
        }
    }

    pub async fn broadcast(&self, message: &Message) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_id in game_state.players.keys().copied() {
            universe.send(player_id, message).await;
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.game_state.lock().await.players.is_empty()
    }
}
