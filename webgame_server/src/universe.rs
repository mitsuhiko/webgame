use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use warp::ws;

use crate::game::Game;
use crate::protocol::{Message, PlayerInfo, ProtocolError, ProtocolErrorKind};
use crate::utils::generate_join_code;

pub struct UniversePlayerState {
    player_info: PlayerInfo,
    is_authenticated: bool,
    game_id: Option<Uuid>,
    tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
}

pub struct UniverseState {
    players: HashMap<Uuid, UniversePlayerState>,
    games: HashMap<Uuid, Arc<Game>>,
    joinable_games: HashMap<String, Uuid>,
}

pub struct Universe {
    state: Arc<RwLock<UniverseState>>,
}

impl Universe {
    pub fn new() -> Universe {
        Universe {
            state: Arc::new(RwLock::new(UniverseState {
                players: HashMap::new(),
                games: HashMap::new(),
                joinable_games: HashMap::new(),
            })),
        }
    }

    /// Starts a new game.
    pub async fn new_game(self: &Arc<Self>) -> Arc<Game> {
        let mut universe_state = self.state.write().await;

        loop {
            let join_code = generate_join_code();
            if universe_state.joinable_games.contains_key(&join_code) {
                continue;
            }

            let game = Arc::new(Game::new(join_code, self.clone()));
            universe_state.games.insert(game.id(), game.clone());
            universe_state
                .joinable_games
                .insert(game.join_code().to_string(), game.id());
            return game;
        }
    }

    /// Joins a player into a game by join code.
    pub async fn join_game(
        &self,
        player_id: Uuid,
        join_code: String,
    ) -> Result<Arc<Game>, ProtocolError> {
        // assign to temporary to release lock.
        let game_id = self
            .state
            .read()
            .await
            .joinable_games
            .get(&join_code)
            .copied();

        if let Some(game_id) = game_id {
            if let Some(game) = self.get_game(game_id).await {
                if game.is_joinable().await {
                    game.add_player(player_id).await;
                    return Ok(game);
                } else {
                    return Err(ProtocolError::new(
                        ProtocolErrorKind::InvalidCommand,
                        "game is currently not joinable",
                    ));
                }
            }
        }

        Err(ProtocolError::new(
            ProtocolErrorKind::NotFound,
            "game does not exist",
        ))
    }

    /// Registers a player.
    ///
    /// The player is given a new ID which is returned and starts out without
    /// any associated nickname.
    pub async fn add_player(
        &self,
        tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
    ) -> Uuid {
        let player_id = Uuid::new_v4();
        let mut universe_state = self.state.write().await;
        universe_state.players.insert(
            player_id,
            UniversePlayerState {
                player_info: PlayerInfo {
                    id: player_id,
                    nickname: "anonymous".into(),
                },
                game_id: None,
                is_authenticated: false,
                tx,
            },
        );
        player_id
    }

    /// Returns the player.
    pub async fn get_player_info(&self, player_id: Uuid) -> Option<PlayerInfo> {
        let universe_state = self.state.read().await;
        universe_state
            .players
            .get(&player_id)
            .map(|x| x.player_info.clone())
    }

    /// Authenticates a player.
    ///
    /// If the user is already authenticated this returns `false`.
    pub async fn authenticate_player(
        &self,
        player_id: Uuid,
        nickname: String,
    ) -> Result<PlayerInfo, ProtocolError> {
        let mut universe_state = self.state.write().await;
        if let Some(player_state) = universe_state.players.get_mut(&player_id) {
            if player_state.is_authenticated {
                Err(ProtocolError::new(
                    ProtocolErrorKind::AlreadyAuthenticated,
                    "cannot authenticate twice",
                ))
            } else {
                player_state.is_authenticated = true;
                player_state.player_info.nickname = nickname;
                Ok(player_state.player_info.clone())
            }
        } else {
            Err(ProtocolError::new(
                ProtocolErrorKind::InternalError,
                "couldn't find user in state",
            ))
        }
    }

    /// Checks if the player is authenticated.
    pub async fn player_is_authenticated(&self, player_id: Uuid) -> bool {
        let universe_state = self.state.read().await;
        if let Some(ref state) = universe_state.players.get(&player_id) {
            state.is_authenticated
        } else {
            false
        }
    }

    /// Unregisters a player.
    pub async fn remove_player(&self, player_id: Uuid) {
        let mut universe_state = self.state.write().await;
        universe_state.players.remove(&player_id);
    }

    /// Sets the current game of a player.
    pub async fn set_player_game_id(&self, player_id: Uuid, game_id: Option<Uuid>) -> bool {
        let mut universe_state = self.state.write().await;
        if let Some(state) = universe_state.players.get_mut(&player_id) {
            state.game_id = game_id;
            true
        } else {
            false
        }
    }

    /// Returns a game by ID
    pub async fn get_game(&self, game_id: Uuid) -> Option<Arc<Game>> {
        let universe_state = self.state.read().await;
        universe_state.games.get(&game_id).cloned()
    }

    /// Removes a game from the universe.
    pub async fn remove_game(&self, game_id: Uuid) -> bool {
        let mut universe_state = self.state.write().await;
        universe_state.games.remove(&game_id).is_some()
    }

    /// Returns the game a player is in.
    pub async fn get_player_game(&self, player_id: Uuid) -> Option<Arc<Game>> {
        let universe_state = self.state.read().await;
        universe_state
            .players
            .get(&player_id)
            .and_then(|player| player.game_id)
            .and_then(|game_id| universe_state.games.get(&game_id))
            .cloned()
    }

    /// Makes the player leave the game they are in.
    pub async fn remove_player_from_game(&self, player_id: Uuid) {
        if let Some(game) = self.get_player_game(player_id).await {
            game.remove_player(player_id).await;
        }
    }

    /// Send a message to a single player.
    pub async fn send(&self, player_id: Uuid, message: &Message) {
        let universe_state = self.state.write().await;
        if let Some(ref state) = universe_state.players.get(&player_id) {
            let s = serde_json::to_string(message).unwrap();
            if let Err(_disconnected) = state.tx.send(Ok(ws::Message::text(s))) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}
