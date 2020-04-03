use std::collections::{BTreeMap, HashSet};
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

    pub async fn set_player_role(&self, player_id: Uuid, role: PlayerRole) {
        let mut game_state = self.game_state.lock().await;
        let mut my_team = None;
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            // if we're not in a team, the player role cannot be set.
            if player_state.team.is_none() {
                return;
            }
            player_state.role = role;
            player_state.ready = false;
            my_team = player_state.team;
        }

        if role == PlayerRole::Spymaster {
            for player_state in game_state.players.values_mut() {
                if player_state.player.id != player_id
                    && player_state.role == PlayerRole::Spymaster
                    && player_state.team == my_team
                {
                    player_state.role = PlayerRole::Operative;
                    player_state.ready = false;
                }
            }
        }
    }

    pub async fn set_player_team(&self, player_id: Uuid, team: Option<Team>) {
        let mut game_state = self.game_state.lock().await;
        let have_spymaster = game_state
            .players
            .values()
            .any(|x| x.role == PlayerRole::Spymaster && x.team == team);

        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.team = team;
            player_state.ready = false;
            if team.is_none() {
                player_state.role = PlayerRole::Spectator;
            } else if have_spymaster {
                player_state.role = PlayerRole::Operative;
            } else {
                player_state.role = PlayerRole::Spymaster;
            }
        }
    }

    pub async fn mark_player_ready(&self, player_id: Uuid) {
        let mut game_state = self.game_state.lock().await;
        if let Some(player_state) = game_state.players.get_mut(&player_id) {
            player_state.ready = true;
        }

        let mut flags = HashSet::new();

        for player in game_state.players.values() {
            match (player.team, player.role) {
                (None, _) | (_, PlayerRole::Spectator) => {}
                (Some(team), role) => {
                    flags.insert((team, role));
                }
            }
        }

        if flags.len() == 4 {
            game_state.turn = game_state.board.initial_turn();
        }
    }

    pub async fn broadcast(&self, message: &Message) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_id in game_state.players.keys().copied() {
            universe.send(player_id, message).await;
        }
    }

    pub async fn broadcast_state(&self) {
        let universe = self.universe();
        let game_state = self.game_state.lock().await;
        for player_id in game_state.players.keys().copied() {
            log::debug!("broadcast game state to {}", player_id);
            let mut players = vec![];
            let mut reveal = false;
            for (&other_player_id, player_state) in game_state.players.iter() {
                if player_id == other_player_id
                    && (player_state.role == PlayerRole::Spymaster
                        || player_state.role == PlayerRole::Spectator)
                    && game_state.turn != Turn::Pregame
                {
                    reveal = true;
                }
                players.push(player_state.clone());
            }
            universe
                .send(
                    player_id,
                    &Message::GameStateSnapshot(GameStateSnapshot {
                        players,
                        tiles: game_state.board.tiles(reveal),
                        turn: game_state.turn,
                    }),
                )
                .await;
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.game_state.lock().await.players.is_empty()
    }
}
