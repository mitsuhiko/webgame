use std::collections::BTreeMap;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;
use warp::{ws, Filter};

use webgame_protocol::{
    AuthenticatedResponse, ChatMessage, GameStateSnapshot, HelloRequest, Message, Packet, Player,
    PlayerConnectedMessage, PlayerDisconnectedMessage, ProtocolError, ProtocolErrorKind, Request,
    Response, SuccessResponse, SendTextRequest,
};

pub struct PlayerState {
    player: Player,
    is_authenticated: bool,
    tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
}

pub struct GameState {
    players: Mutex<BTreeMap<Uuid, PlayerState>>,
}

impl GameState {
    pub async fn snapshot(&self) -> GameStateSnapshot {
        GameStateSnapshot {
            players: self
                .players
                .lock()
                .await
                .iter()
                .map(|(_id, player_state)| player_state.player.clone())
                .collect(),
        }
    }
}

async fn send(player_id: Uuid, packet: Packet, state: Arc<GameState>) {
    if let Some(ref state) = state.players.lock().await.get(&player_id) {
        let s = serde_json::to_string(&packet).unwrap();
        if let Err(_disconnected) = state.tx.send(Ok(ws::Message::text(s))) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn broadcast(message: Message, state: Arc<GameState>) {
    let s = serde_json::to_string(&Packet::Message(message)).unwrap();
    for (_, ref state) in state.players.lock().await.iter_mut() {
        if !state.is_authenticated {
            continue;
        }
        if let Err(_disconnected) = state.tx.send(Ok(ws::Message::text(s.clone()))) {
            // The tx is disconnected, our `user_disconnected` code
            // should be happening in another task, nothing more to
            // do here.
        }
    }
}

async fn on_player_connected(state: Arc<GameState>, ws: ws::WebSocket) {
    let player_id = Uuid::new_v4();

    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            log::error!("websocket send error: {}", e);
        }
    }));

    state.players.lock().await.insert(
        player_id,
        PlayerState {
            player: Player {
                id: player_id,
                nickname: "<unknown>".into(),
            },
            is_authenticated: false,
            tx,
        },
    );
    log::info!("user {:#?} connected", player_id);

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                let resp = match on_player_message(player_id, msg, state.clone()).await {
                    Ok(resp) => Response::Ok(resp),
                    Err(err) => Response::Error(err),
                };
                send(player_id, Packet::Response(resp), state.clone()).await;
            }
            Err(e) => {
                log::error!("websocket error(uid={}): {}", player_id, e);
                break;
            }
        }
    }

    on_player_disconnected(player_id, state).await;
}

async fn on_player_disconnected(player_id: Uuid, state: Arc<GameState>) {
    state.players.lock().await.remove(&player_id);
    log::info!("user {:#?} disconnected", player_id);
    broadcast(
        Message::PlayerDisconnected(PlayerDisconnectedMessage { player_id }),
        state,
    )
    .await;
}

async fn on_player_message(
    player_id: Uuid,
    msg: ws::Message,
    state: Arc<GameState>,
) -> Result<SuccessResponse, ProtocolError> {
    let req_json = match msg.to_str() {
        Ok(text) => text,
        Err(()) => {
            return Err(ProtocolError::new(
                ProtocolErrorKind::InvalidRequest,
                "not a valid text frame",
            ))
        }
    };

    let req: Request = match serde_json::from_str(&req_json) {
        Ok(req) => req,
        Err(err) => {
            return Err(ProtocolError::new(
                ProtocolErrorKind::InvalidRequest,
                err.to_string(),
            ));
        }
    };

    match req {
        Request::Hello(data) => on_player_hello(player_id, data, state).await,
        Request::SendText(data) => on_player_send_text(player_id, data, state).await,
        Request::RefreshGameState => Ok(SuccessResponse::GameStateSnapshot(state.snapshot().await)),
    }
}

async fn on_player_hello(
    player_id: Uuid,
    req: HelloRequest,
    state: Arc<GameState>,
) -> Result<SuccessResponse, ProtocolError> {
    if let Some(ref mut player_state) = state.players.lock().await.get_mut(&player_id) {
        if player_state.is_authenticated {
            return Err(ProtocolError::new(
                ProtocolErrorKind::AlreadyAuthenticated,
                "cannot authenticate twice",
            ));
        }
        player_state.is_authenticated = true;
        player_state.player.nickname = req.nickname.clone();
    } else {
        return Err(ProtocolError::new(
            ProtocolErrorKind::InternalError,
            "couldn't find user in state",
        ));
    }
    log::info!("user {:?} said hello as {:?}", player_id, &req.nickname);
    broadcast(
        Message::PlayerConnected(PlayerConnectedMessage {
            player_id,
            nickname: req.nickname,
        }),
        state.clone(),
    )
    .await;
    Ok(SuccessResponse::Authenticated(AuthenticatedResponse {
        player_id,
    }))
}

pub async fn on_player_send_text(
    player_id: Uuid,
    req: SendTextRequest,
    state: Arc<GameState>,
) -> Result<SuccessResponse, ProtocolError> {
    let msg = Message::Chat(ChatMessage {
        player_id,
        text: req.text,
    });
    broadcast(msg, state).await;
    Ok(SuccessResponse::Ack)
}

pub async fn serve() {
    let state = Arc::new(GameState {
        players: Mutex::new(BTreeMap::new()),
    });

    let routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || state.clone()))
        .map(|ws: warp::ws::Ws, state: Arc<GameState>| {
            ws.on_upgrade(move |ws| on_player_connected(state, ws))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
