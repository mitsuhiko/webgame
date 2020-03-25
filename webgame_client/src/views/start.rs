use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
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
    error: Option<String>,
}

pub enum Msg {
    Authenticate,
    ServerMessage(Message),
    SetNickname(String),
    Ignore,
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
            nickname: "".into(),
            on_authenticate: props.on_authenticate,
            error: None,
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
                    Message::Error(err) => {
                        self.error = Some(err.message().to_string());
                    }
                    _ => {}
                }
            }
            Msg::SetNickname(nickname) => {
                self.nickname = nickname;
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{"Whatever Together"}</h1>
                <p class="explanation">
                    {"Give yourself a name to play:"}
                </p>
                <div class="toolbar">
                    <input value=&self.nickname
                        placeholder="nickname"
                        onkeypress=self.link.callback(|event: KeyboardEvent| {
                            dbg!(event.key());
                            if event.key() == "Enter" {
                                Msg::Authenticate
                            } else {
                                Msg::Ignore
                            }
                        })
                        oninput=self.link.callback(|e: InputData| Msg::SetNickname(e.value)) />
                    <button
                        onclick=self.link.callback(|_| Msg::Authenticate)>{"Play"}</button>
                    {
                        if let Some(ref error) = self.error {
                            html! {
                                <p class="error">{format!("not good: {}", error)}</p>
                            }
                        } else {
                            html!{}
                        }
                    }
                </div>
            </div>
        }
    }
}
