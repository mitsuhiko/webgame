use std::rc::Rc;

use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::{GameStateSnapshot, PlayerRole, Team, Turn};

#[derive(Clone, Properties)]
pub struct Props {
    pub game_state: Rc<GameStateSnapshot>,
}

pub struct PlayerList {
    game_state: Rc<GameStateSnapshot>,
}

impl Component for PlayerList {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        PlayerList {
            game_state: props.game_state,
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.game_state != props.game_state {
            self.game_state = props.game_state;
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
                    self.game_state.players.iter().map(|state| html! {
                        <li class={
                            match state.team {
                                None => "neutral",
                                Some(Team::Red) => "team-red",
                                Some(Team::Blue) => "team-blue",
                            }
                        }>
                            <span class="nickname">{&state.player.nickname}</span>
                            {format!(
                                " {}",
                                match state.role {
                                    PlayerRole::Spymaster => "(Spymaster)",
                                    PlayerRole::Operative => "",
                                    PlayerRole::Spectator => "(Spectator)",
                                }
                            )}
                            {
                                if self.game_state.turn == Turn::Pregame &&
                                    state.ready {
                                    html! { " — ready" }
                                } else {
                                    html!{}
                                }
                            }
                        </li>
                    }).collect::<Html>()
                }
                </ul>
            </div>
        }
    }
}
