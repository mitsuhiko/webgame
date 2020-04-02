use std::rc::Rc;

use im_rc::HashMap;
use uuid::Uuid;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::{GamePlayerState, PlayerRole, Team};

#[derive(Clone, Properties)]
pub struct Props {
    pub players: HashMap<Uuid, Rc<GamePlayerState>>,
}

pub struct PlayerList {
    players: HashMap<Uuid, Rc<GamePlayerState>>,
}

impl Component for PlayerList {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        PlayerList {
            players: props.players,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.players != props.players {
            self.players = props.players;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="players box">
                <h2>{"Players"}</h2>
                <ul>
                {
                    self.players.iter().map(|(_, state)| html! {
                        <li class={
                            match state.team {
                                None => "neutral",
                                Some(Team::Red) => "team-red",
                                Some(Team::Blue) => "team-blue",
                            }
                        }>
                            {format!(
                                "{} ({})",
                                state.player.nickname,
                                match state.role {
                                    PlayerRole::Spymaster => "Spymaster",
                                    PlayerRole::Operative => "Operative",
                                    PlayerRole::Spectator => "Spectator",
                                }
                            )}
                        </li>
                    }).collect::<Html>()
                }
                </ul>
            </div>
        }
    }
}
