use std::convert::Infallible;
use std::sync::Arc;

use futures::{FutureExt, StreamExt};
use hyper::{service::make_service_fn, Server};
use tokio::sync::mpsc;
use uuid::Uuid;
use warp::{ws, Filter};

use crate::protocol::{
    AuthenticateCommand, ChatMessage, Command, JoinGameCommand, Message, ProtocolError,
    ProtocolErrorKind, SendTextCommand, SetPlayerRoleCommand, SetPlayerTeamCommand,
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
                if let Err(err) = on_player_message(universe.clone(), player_id, msg).await {
                    universe.send(player_id, &Message::Error(err)).await;
                }
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
) -> Result<(), ProtocolError> {
    let req_json = match msg.to_str() {
        Ok(text) => text,
        Err(()) => {
            return Err(ProtocolError::new(
                ProtocolErrorKind::InvalidCommand,
                "not a valid text frame",
            ))
        }
    };

    let cmd: Command = match serde_json::from_str(&req_json) {
        Ok(req) => req,
        Err(err) => {
            return Err(ProtocolError::new(
                ProtocolErrorKind::InvalidCommand,
                err.to_string(),
            ));
        }
    };

    log::debug!("command: {:?}", &cmd);

    if !universe.player_is_authenticated(player_id).await {
        match cmd {
            Command::Authenticate(data) => on_player_authenticate(universe, player_id, data).await,
            _ => Err(ProtocolError::new(
                ProtocolErrorKind::NotAuthenticated,
                "cannot perform this command unauthenticated",
            )),
        }
    } else {
        match cmd {
            Command::NewGame => on_new_game(universe, player_id).await,
            Command::JoinGame(data) => on_join_game(universe, player_id, data).await,
            Command::LeaveGame => on_leave_game(universe, player_id).await,
            Command::MarkReady => on_player_mark_ready(universe, player_id).await,
            Command::SendText(data) => on_player_send_text(universe, player_id, data).await,
            Command::SetPlayerRole(data) => on_player_set_role(universe, player_id, data).await,
            Command::SetPlayerTeam(data) => on_player_set_team(universe, player_id, data).await,

            // this should not happen here.
            Command::Authenticate(..) => Err(ProtocolError::new(
                ProtocolErrorKind::AlreadyAuthenticated,
                "cannot authenticate twice",
            )),
        }
    }
}

async fn on_new_game(universe: Arc<Universe>, player_id: Uuid) -> Result<(), ProtocolError> {
    universe.remove_player_from_game(player_id).await;
    let game = universe.new_game().await;
    game.add_player(player_id).await;
    universe
        .send(player_id, &Message::GameJoined(game.game_info()))
        .await;
    game.broadcast_state().await;
    Ok(())
}

async fn on_join_game(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: JoinGameCommand,
) -> Result<(), ProtocolError> {
    let game = universe.join_game(player_id, cmd.join_code).await?;
    universe
        .send(player_id, &Message::GameJoined(game.game_info()))
        .await;
    game.broadcast_state().await;
    Ok(())
}

async fn on_leave_game(universe: Arc<Universe>, player_id: Uuid) -> Result<(), ProtocolError> {
    universe.remove_player_from_game(player_id).await;
    universe.send(player_id, &Message::GameLeft).await;
    Ok(())
}

async fn on_player_authenticate(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: AuthenticateCommand,
) -> Result<(), ProtocolError> {
    let nickname = cmd.nickname.trim().to_owned();
    if nickname.is_empty() || nickname.len() > 16 {
        return Err(ProtocolError::new(
            ProtocolErrorKind::BadInput,
            "nickname must be between 1 and 16 characters",
        ));
    }

    let player_info = universe.authenticate_player(player_id, nickname).await?;
    log::info!(
        "player {:?} authenticated as {:?}",
        player_id,
        &player_info.nickname
    );

    universe
        .send(player_id, &Message::Authenticated(player_info.clone()))
        .await;

    Ok(())
}

pub async fn on_player_mark_ready(
    universe: Arc<Universe>,
    player_id: Uuid,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        if game.is_joinable().await {
            game.mark_player_ready(player_id).await;
            game.broadcast_state().await;
        }
    }
    Ok(())
}

pub async fn on_player_send_text(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: SendTextCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        game.broadcast(&Message::Chat(ChatMessage {
            player_id,
            text: cmd.text,
        }))
        .await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_set_role(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: SetPlayerRoleCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        if !game.is_joinable().await {
            return Err(ProtocolError::new(
                ProtocolErrorKind::BadState,
                "cannot set role because game is not not joinable",
            ));
        }
        game.set_player_role(player_id, cmd.role).await;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn on_player_set_team(
    universe: Arc<Universe>,
    player_id: Uuid,
    cmd: SetPlayerTeamCommand,
) -> Result<(), ProtocolError> {
    if let Some(game) = universe.get_player_game(player_id).await {
        if !game.is_joinable().await {
            return Err(ProtocolError::new(
                ProtocolErrorKind::BadState,
                "cannot set team because game is not not joinable",
            ));
        }
        game.set_player_team(player_id, cmd.team).await;
        game.broadcast_state().await;
        Ok(())
    } else {
        Err(ProtocolError::new(
            ProtocolErrorKind::BadState,
            "not in a game",
        ))
    }
}

pub async fn serve() {
    let universe = Arc::new(Universe::new());

    let make_svc = make_service_fn(move |_| {
        let universe = universe.clone();
        let routes = warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map(move || universe.clone()))
            .map(|ws: warp::ws::Ws, universe: Arc<Universe>| {
                ws.on_upgrade(move |ws| on_player_connected(universe, ws))
            });
        let svc = warp::service(routes);
        async move { Ok::<_, Infallible>(svc) }
    });

    let mut listenfd = listenfd::ListenFd::from_env();
    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        Server::from_tcp(l).unwrap()
    } else {
        Server::bind(&([127, 0, 0, 1], 8002).into())
    };
    server.serve(make_svc).await.unwrap();
}
