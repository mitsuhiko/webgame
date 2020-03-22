use uuid::Uuid;

use yew::format::Json;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew::{html, Component, ComponentLink, Html, InputData, ShouldRender};

use webgame_protocol::{AuthenticateRequest, Packet, Request, Response, SuccessResponse};

pub struct Model {
    link: ComponentLink<Self>,
    ws: Option<WebSocketTask>,
    ws_service: WebSocketService,
    nickname: String,
    player_id: Uuid,
}

pub enum Msg {
    Connect,
    Authenticate,
    Packet(Packet),
    Ignore,
    ConnectionLost,
    SetNickname(String),
}

impl Model {
    pub fn send_request(&mut self, req: Request) {
        self.ws.as_mut().unwrap().send(Json(&req));
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Model {
            link,
            ws: None,
            ws_service: WebSocketService::new(),
            nickname: "anonymous".into(),
            player_id: Uuid::nil(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Connect => {
                log::info!("Connecting to game");
                let on_message = self.link.callback(|Json(data)| match data {
                    Ok(packet) => Msg::Packet(packet),
                    Err(err) => {
                        log::error!("websocket error: {:?}", err);
                        Msg::Ignore
                    }
                });
                let on_notification = self.link.callback(|status| match status {
                    WebSocketStatus::Opened => Msg::Authenticate,
                    WebSocketStatus::Closed | WebSocketStatus::Error => Msg::ConnectionLost,
                });
                self.ws = Some(
                    self.ws_service
                        .connect("ws://127.0.0.1:8002/ws", on_message, on_notification)
                        .unwrap(),
                );
            }
            Msg::Authenticate => {
                self.send_request(Request::Authenticate(AuthenticateRequest {
                    nickname: self.nickname.clone(),
                }));
            }
            Msg::Packet(Packet::Message(message)) => {
                log::info!("message: {:?}", message);
            }
            Msg::Packet(Packet::Response(Response::Ok(result))) => {
                log::info!("ok response: {:?}", result);
                match result {
                    SuccessResponse::Authenticated(data) => {
                        self.player_id = data.player_id;
                    }
                    _ => {}
                }
            }
            Msg::Packet(Packet::Response(Response::Error(error))) => {
                log::error!("request resulted in an error: {:?}", error);
            }
            Msg::Ignore => {}
            Msg::ConnectionLost => {}
            Msg::SetNickname(nickname) => {
                self.nickname = nickname;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <input value=&self.nickname
                oninput=self.link.callback(|e: InputData| Msg::SetNickname(e.value)) />
                <button onclick=self.link.callback(|_| Msg::Connect)>{"Connect"}</button>
            </div>
        }
    }
}
