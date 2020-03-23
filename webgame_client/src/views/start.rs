use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, Properties, ShouldRender,
};

use crate::api::Api;
use crate::protocol::{AuthenticateCommand, Command, Message, PlayerInfo};

#[derive(Clone, PartialEq, Properties)]
pub struct Props {
    pub on_authenticate: Callback<PlayerInfo>,
}

pub struct StartPage {
    link: ComponentLink<StartPage>,
    api: Box<dyn Bridge<Api>>,
    nickname: String,
    on_authenticate: Callback<PlayerInfo>,
}

pub enum Msg {
    Authenticate,
    ServerMessage(Message),
    SetNickname(String),
}

impl Component for StartPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        StartPage {
            link,
            api,
            nickname: "anonymous".into(),
            on_authenticate: props.on_authenticate,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticate => {
                log::info!("Authenticating");
                self.api.send(Command::Authenticate(AuthenticateCommand {
                    nickname: self.nickname.clone(),
                }));
            }
            Msg::ServerMessage(message) => {
                log::info!("message: {:?}", message);
                match message {
                    Message::Authenticated(data) => {
                        self.on_authenticate.emit(data);
                    }
                    _ => {}
                }
            }
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
                <button onclick=self.link.callback(|_| Msg::Authenticate)>{"Play"}</button>
            </div>
        }
    }
}
