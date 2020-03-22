use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::{ws, Filter};

use crate::protocol::{
    AuthenticateRequest, AuthenticatedResponse, ChatMessage, GameJoinedResponse, JoinGameRequest,
    Message, NewGameResponse, Packet, PlayerConnectedMessage, ProtocolError, ProtocolErrorKind,
    Request, Response, SendTextRequest, SuccessResponse,
};
use crate::universe::Universe;

async fn on_player_connected(universe: Arc<Universe>, ws: ws::WebSocket) {
    let (user_ws_tx, mut user_ws_rx) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();

    tokio::task::spawn(rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            log::error!("websocket send error: {}", e);
        }
    }));

    let player_id = universe.add_player(tx).await;
    log::info!("player {:#?} connected", player_id);

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => {
                log::debug!("Got message from websocket: {:?}", &msg);
                let resp = match on_player_message(universe.clone(), player_id, msg).await {
                    Ok(resp) => Response::Ok(resp),
                    Err(err) => Response::Error(err),
                };
                universe.send(player_id, &Packet::Response(resp)).await;
            }
            Err(e) => {
                log::error!("websocket error(uid={}): {}", player_id, e);
                break;
            }
        }
    }

    on_player_disconnected(universe, player_id).await;
}

async fn on_player_disconnected(universe: Arc<Universe>, player_id: Uuid) {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.remove_player(player_id).await;
    }
    universe.remove_player(player_id).await;
    log::info!("user {:#?} disconnected", player_id);
}

async fn on_player_message(
    universe: Arc<Universe>,
    player_id: Uuid,
    msg: ws::Message,
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

    if !universe.player_is_authenticated(player_id).await {
        match req {
            Request::Authenticate(data) => on_player_authenticate(universe, player_id, data).await,
            _ => Err(ProtocolError::new(
                ProtocolErrorKind::NotAuthenticated,
                "cannot perform this command unauthenticated",
            )),
        }
    } else {
        match req {
            Request::NewGame => on_new_game(universe, player_id).await,
            Request::JoinGame(data) => on_join_game(universe, player_id, data).await,
            Request::SendText(data) => on_player_send_text(universe, player_id, data).await,
            Request::MarkReady => on_player_mark_ready(universe, player_id).await,

            // this should not happen here.
            Request::Authenticate(..) => Err(ProtocolError::new(
                ProtocolErrorKind::AlreadyAuthenticated,
                "cannot authenticate twice",
            )),
        }
    }
}

async fn on_new_game(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<SuccessResponse, ProtocolError> {
    universe.remove_player_from_game(player_id).await;
    let (game, join_code) = universe.new_game().await;
    game.add_player(player_id).await;
    Ok(SuccessResponse::NewGame(NewGameResponse {
        game_id: game.id(),
        join_code,
    }))
}

async fn on_join_game(
    universe: Arc<Universe>,
    player_id: Uuid,
    req: JoinGameRequest,
) -> Result<SuccessResponse, ProtocolError> {
    if let Some(game) = universe.join_game(player_id, req.join_code).await {
        Ok(SuccessResponse::GameJoined(GameJoinedResponse {
            game_id: game.id(),
        }))
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::NotFound,
            "game was not found",
        ))
    }
}

async fn on_player_authenticate(
    universe: Arc<Universe>,
    player_id: Uuid,
    req: AuthenticateRequest,
) -> Result<SuccessResponse, ProtocolError> {
    let player_info = universe
        .authenticate_player(player_id, req.nickname)
        .await?;
    log::info!(
        "player {:?} authenticated as {:?}",
        player_id,
        &player_info.nickname
    );
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Packet::Message(Message::PlayerConnected(
            PlayerConnectedMessage { player_info },
        )))
        .await;
    }
    Ok(SuccessResponse::Authenticated(AuthenticatedResponse {
        player_id,
    }))
}

pub async fn on_player_send_text(
    universe: Arc<Universe>,
    player_id: Uuid,
    req: SendTextRequest,
) -> Result<SuccessResponse, ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Packet::Message(Message::Chat(ChatMessage {
            player_id,
            text: req.text,
        })))
        .await;
        Ok(SuccessResponse::Ack)
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::InvalidRequest,
            "not in a game",
        ))
    }
}

pub async fn on_player_mark_ready(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<SuccessResponse, ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.mark_player_ready(player_id).await;
        Ok(SuccessResponse::Ack)
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::InvalidRequest,
            "not in a game",
        ))
    }
}

pub async fn serve() {
    let universe = Arc::new(Universe::new());

    let routes = warp::path("ws")
        .and(warp::ws())
        .and(warp::any().map(move || universe.clone()))
        .map(|ws: warp::ws::Ws, universe: Arc<Universe>| {
            ws.on_upgrade(move |ws| on_player_connected(universe, ws))
        });

    warp::serve(routes).run(([127, 0, 0, 1], 8002)).await;
}
