use std::mem;
use std::rc::Rc;

use im_rc::{HashMap, Vector};
use uuid::Uuid;
use yew::agent::Bridged;
use yew::{
    html, Bridge, Callback, Component, ComponentLink, Html, InputData, KeyboardEvent, Properties,
    ShouldRender,
};

use crate::api::Api;
use crate::components::chat_box::{ChatBox, ChatLine, ChatLineData};
use crate::components::player_list::PlayerList;
use crate::protocol::{Character, Command, GameInfo, Message, PlayerInfo, SendTextCommand, Tile};
use crate::utils::format_join_code;

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
    players: HashMap<Uuid, Rc<PlayerInfo>>,
    tiles: Vec<Tile>,
    chat_line: String,
    chat_log: Vector<Rc<ChatLine>>,
    on_game_command: Callback<GamePageCommand>,
}

pub enum Msg {
    Ignore,
    SendChat,
    SetChatLine(String),
    ServerMessage(Message),
}

impl GamePage {
    pub fn add_chat_message(&mut self, player_id: Uuid, data: ChatLineData) {
        let nickname = self
            .players
            .get(&player_id)
            .map(|x| x.nickname.as_str())
            .unwrap_or("anonymous")
            .to_string();
        self.chat_log
            .push_back(Rc::new(ChatLine { nickname, data }));
        while self.chat_log.len() > 20 {
            self.chat_log.pop_front();
        }
    }
}

fn get_tile_class(tile: &Tile) -> &'static str {
    match tile.character {
        Character::BlueAgent => "tile blue-agent",
        Character::RedAgent => "tile red-agent",
        Character::Bystander => "tile bystander",
        Character::Assassin => "tile assassin",
        Character::Unknown => "tile unspotted",
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
            chat_line: "".into(),
            chat_log: Vector::unit(Rc::new(ChatLine {
                nickname: props.player_info.nickname.clone(),
                data: ChatLineData::Connected,
            })),
            tiles: Vec::new(),
            players: HashMap::new(),
            player_info: props.player_info,
            on_game_command: props.on_game_command,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(message) => match message {
                Message::Chat(msg) => {
                    self.add_chat_message(msg.player_id, ChatLineData::Text(msg.text));
                }
                Message::PlayerConnected(info) => {
                    let player_id = info.id;
                    self.players.insert(player_id, Rc::new(info));
                    self.add_chat_message(player_id, ChatLineData::Connected);
                }
                Message::PlayerDisconnected(msg) => {
                    self.add_chat_message(msg.player_id, ChatLineData::Disconnected);
                    self.players.remove(&msg.player_id);
                }
                Message::GameStateSnapshot(snapshot) => {
                    self.tiles = snapshot.tiles;
                    self.players = snapshot
                        .players
                        .into_iter()
                        .map(|x| (x.id, Rc::new(x)))
                        .collect();
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
            Msg::Ignore => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{format!("Game ({})", format_join_code(&self.game_info.join_code))}</h1>
                <div class="box tiles">
                {
                    self.tiles.iter().map(|tile| html! {
                        <div class={get_tile_class(tile)}>
                            <span>{&tile.codeword}</span>
                        </div>
                    }).collect::<Html>()
                }
                </div>
                <PlayerList players=self.players.clone()/>
                <ChatBox log=self.chat_log.clone()/>
                <div class="toolbar">
                    <span>{format!("{}: ", &self.player_info.nickname)}</span>
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
                    <button class="primary" onclick=self.link.callback(|_| Msg::SendChat)>{"Send"}</button>
                </div>
                <div class="toolbar">
                    <button>{"Red Team"}</button>
                    <button>{"Blue Team"}</button>
                    <button>{"Become Spymaster"}</button>
                    <button>{"Become Operative"}</button>
                    <button>{"Shuffle Board"}</button>
                </div>
            </div>
        }
    }
}
