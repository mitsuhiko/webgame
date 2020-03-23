use yew::agent::Bridged;
use yew::{html, Bridge, Callback, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::api::Api;
use crate::protocol::{Command, GameInfo, Message, PlayerInfo};

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
    on_game_command: Callback<GamePageCommand>,
}

pub enum Msg {
    Disconnect,
    ServerMessage(Message),
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
            on_game_command: props.on_game_command,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::ServerMessage(_message) => {}
            Msg::Disconnect => {
                self.api.send(Command::LeaveGame);
                self.on_game_command.emit(GamePageCommand::Quit);
            }
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <h1>{"In Game!"}</h1>
                <pre>{format!("{:#?}", &self.game_info)}</pre>
                <pre>{format!("{:#?}", &self.player_info)}</pre>
                <button onclick=self.link.callback(|_| Msg::Disconnect)>{"Disconnect"}</button>
            </div>
        }
    }
}
