use std::collections::HashSet;

use yew::agent::{Agent, AgentLink, Context, HandlerId};
use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

use crate::protocol::{Command, Message};

#[derive(Debug)]
pub enum ApiState {
    Connecting,
    Connected,
    Disconnected,
}

pub enum Msg {
    ServerMessage(Message),
    Connected,
    ConnectionLost,
    Ignore,
}

#[derive(Debug)]
pub struct Api {
    link: AgentLink<Api>,
    ws: WebSocketTask,
    ws_service: WebSocketService,
    subscribers: HashSet<HandlerId>,
    state: ApiState,
}

fn get_websocket_location() -> String {
    format!(
        "ws://{}/ws",
        web_sys::window().unwrap().location().host().unwrap()
    )
}

impl Agent for Api {
    type Reach = Context;
    type Message = Msg;
    type Input = Command;
    type Output = Message;

    fn create(link: AgentLink<Api>) -> Api {
        log::info!("Connecting to server");
        let on_message = link.callback(|Json(data)| match data {
            Ok(message) => Msg::ServerMessage(message),
            Err(err) => {
                log::error!("websocket error: {:?}", err);
                Msg::Ignore
            }
        });
        let on_notification = link.callback(|status| match status {
            WebSocketStatus::Opened => Msg::Connected,
            WebSocketStatus::Closed | WebSocketStatus::Error => Msg::ConnectionLost,
        });
        let mut ws_service = WebSocketService::new();
        let ws = ws_service
            .connect(&get_websocket_location(), on_message, on_notification)
            .unwrap();

        Api {
            link,
            ws,
            ws_service,
            state: ApiState::Connecting,
            subscribers: HashSet::new(),
        }
    }

    fn handle_input(&mut self, input: Self::Input, _: HandlerId) {
        log::debug!("Sending command: {:?}", &input);
        self.ws.send(Json(&input));
    }

    fn update(&mut self, msg: Self::Message) {
        match msg {
            Msg::ServerMessage(msg) => {
                log::debug!("Server message: {:?}", msg);
                for sub in self.subscribers.iter() {
                    self.link.respond(*sub, msg.clone());
                }
            }
            Msg::Connected => {
                log::info!("Connected web socket!");
                self.state = ApiState::Connected;
            }
            Msg::ConnectionLost => {
                log::info!("Lost connection on web socket!");
                self.state = ApiState::Disconnected;
            }
            Msg::Ignore => {}
        }
    }

    fn connected(&mut self, id: HandlerId) {
        self.subscribers.insert(id);
    }

    fn disconnected(&mut self, id: HandlerId) {
        self.subscribers.remove(&id);
    }

    fn destroy(&mut self) {
        log::info!("destroying API service");
    }
}
