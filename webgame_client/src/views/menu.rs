use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, Properties, ShouldRender,
};

use crate::api::Api;
use crate::protocol::{Command, GameInfo, JoinGameCommand, Message, PlayerInfo};

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub on_game_joined: Callback<GameInfo>,
}

pub struct MenuPage {
    link: ComponentLink<MenuPage>,
    api: Box<dyn Bridge<Api>>,
    join_code: String,
    player_info: PlayerInfo,
    on_game_joined: Callback<GameInfo>,
    error: Option<String>,
}

pub enum Msg {
    NewGame,
    JoinGame,
    ServerMessage(Message),
    SetJoinCode(String),
}

impl Component for MenuPage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        MenuPage {
            link,
            api,
            join_code: "".into(),
            player_info: props.player_info,
            on_game_joined: props.on_game_joined,
            error: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::NewGame => {
                log::info!("New Game");
                self.api.send(Command::NewGame);
            }
            Msg::JoinGame => {
                log::info!("Join Game");
                self.api.send(Command::JoinGame(JoinGameCommand {
                    join_code: self.join_code.clone(),
                }));
            }
            Msg::ServerMessage(message) => match message {
                Message::GameJoined(data) => {
                    self.on_game_joined.emit(data);
                }
                Message::Error(err) => {
                    self.error = Some(err.message().to_string());
                }
                _ => {}
            },
            Msg::SetJoinCode(join_code) => {
                self.join_code = join_code;
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <p>{format!("Hello {} [{}]", &self.player_info.nickname, &self.player_info.id)}</p>
                <button onclick=self.link.callback(|_| Msg::NewGame)>{"New Game"}</button>
                <input value=&self.join_code
                    oninput=self.link.callback(|e: InputData| Msg::SetJoinCode(e.value)) />
                <button onclick=self.link.callback(|_| Msg::JoinGame)>{"Join Game"}</button>
                {
                    if let Some(ref error) = self.error {
                        html! {
                            <p>{format!("uh oh: {}", error)}</p>
                        }
                    } else {
                        html!{}
                    }
                }
            </div>
        }
    }
}
