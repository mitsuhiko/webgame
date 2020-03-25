use std::collections::VecDeque;
use std::mem;

use uuid::Uuid;
use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
};

use crate::api::Api;
use crate::protocol::{Command, GameInfo, Message, PlayerInfo, SendTextCommand};

#[derive(Clone, Debug)]
pub enum GamePageCommand {
    Quit,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub player_info: PlayerInfo,
    pub game_info: GameInfo,
    pub on_game_command: Callback<GamePageCommand>,
}

pub struct GamePage {
    link: ComponentLink<GamePage>,
    api: Box<dyn Bridge<Api>>,
    game_info: GameInfo,
    player_info: PlayerInfo,
    chat_line: String,
    chat_log: VecDeque<String>,
    on_game_command: Callback<GamePageCommand>,
}

pub enum Msg {
    Ignore,
    Disconnect,
    SendChat,
    SetChatLine(String),
    ServerMessage(Message),
}

impl GamePage {
    pub fn add_chat_message(&mut self, player_id: Uuid, text: &str) {
        self.chat_log.push_back(format!("<{}> {}", player_id, text));
        while self.chat_log.len() > 20 {
            self.chat_log.pop_front();
        }
    }
}

impl Component for GamePage {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        let api = Api::bridge(on_server_message);
        GamePage {
            link,
            api,
            game_info: props.game_info,
            player_info: props.player_info,
            chat_line: "".into(),
            chat_log: VecDeque::new(),
            on_game_command: props.on_game_command,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(message) => match message {
                Message::Chat(msg) => {
                    self.add_chat_message(msg.player_id, &msg.text);
                }
                Message::PlayerConnected(info) => {
                    self.add_chat_message(info.id, "*connected*");
                }
                Message::PlayerDisconnected(msg) => {
                    self.add_chat_message(msg.player_id, "*disconnected*");
                }
                _ => {}
            },
            Msg::SendChat => {
                let text = mem::replace(&mut self.chat_line, "".into());
                self.api.send(Command::SendText(SendTextCommand { text }));
            }
            Msg::SetChatLine(text) => {
                self.chat_line = text;
            }
            Msg::Disconnect => {
                self.api.send(Command::LeaveGame);
                self.on_game_command.emit(GamePageCommand::Quit);
            }
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{"In Game!"}</h1>
                <pre>{format!("{:#?}", &self.game_info)}</pre>
                <pre>{format!("{:#?}", &self.player_info)}</pre>
                <pre>{self.chat_log
                    .iter()
                    .map(|x| format!("{}\n", x))
                    .collect::<String>()}</pre>
                <div class="toolbar">
                    <input value=&self.chat_line
                        placeholder="send some text"
                        size="30"
                        onkeypress=self.link.callback(|event: KeyboardEvent| {
                            if event.key() == "Enter" {
                                Msg::SendChat
                            } else {
                                Msg::Ignore
                            }
                        })
                        oninput=self.link.callback(|e: InputData| Msg::SetChatLine(e.value)) />
                    <button onclick=self.link.callback(|_| Msg::SendChat)>{"Send"}</button>
                </div>
                <button onclick=self.link.callback(|_| Msg::Disconnect)>{"Disconnect"}</button>
            </div>
        }
    }
}
