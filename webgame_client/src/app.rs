use yew::agent::Bridged;
use yew::{html, Bridge, Component, ComponentLink, Html, ShouldRender};

use crate::api::Api;
use crate::protocol::{GameInfo, Message, PlayerInfo};
use crate::views::game::{GamePage, GamePageCommand};
use crate::views::menu::MenuPage;
use crate::views::start::StartPage;

pub struct App {
    _api: Box<dyn Bridge<Api>>,
    link: ComponentLink<Self>,
    state: AppState,
    player_info: Option<PlayerInfo>,
    game_info: Option<GameInfo>,
}

#[derive(Debug)]
enum AppState {
    Start,
    Authenticated,
    InGame,
}

pub enum Msg {
    Authenticated(PlayerInfo),
    GameJoined(GameInfo),
    GamePageCommand(GamePageCommand),
    ServerMessage(Message),
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let on_server_message = link.callback(Msg::ServerMessage);
        App {
            link,
            _api: Api::bridge(on_server_message),
            state: AppState::Start,
            player_info: None,
            game_info: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Authenticated(player_info) => {
                self.state = AppState::Authenticated;
                self.player_info = Some(player_info);
            }
            Msg::GameJoined(game_info) => {
                self.state = AppState::InGame;
                self.game_info = Some(game_info);
            }
            Msg::ServerMessage(Message::GameLeft) | Msg::GamePageCommand(GamePageCommand::Quit) => {
                self.state = AppState::Authenticated;
                self.game_info = None;
            }
            Msg::ServerMessage(_) => {}
        }
        true
    }

    fn view(&self) -> Html {
        html! {
            <div class="game">
            {match self.state {
                AppState::Start => html! {
                    <StartPage on_authenticate=self.link.callback(Msg::Authenticated) />
                },
                AppState::Authenticated => html! {
                    <MenuPage
                        player_info=self.player_info.as_ref().unwrap().clone(),
                        on_game_joined=self.link.callback(Msg::GameJoined) />
                },
                AppState::InGame => html! {
                    <GamePage
                        player_info=self.player_info.as_ref().unwrap().clone(),
                        game_info=self.game_info.as_ref().unwrap().clone(),
                        on_game_command=self.link.callback(Msg::GamePageCommand) />
                }
            }}
            </div>
        }
    }
}
