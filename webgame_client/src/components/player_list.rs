use std::rc::Rc;

use im_rc::HashMap;
use uuid::Uuid;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

use crate::protocol::PlayerInfo;

#[derive(Clone, Properties)]
pub struct Props {
    pub players: HashMap<Uuid, Rc<PlayerInfo>>,
}

pub struct PlayerList {
    players: HashMap<Uuid, Rc<PlayerInfo>>,
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
                    self.players.iter().map(|(_, info)| html! {
                        <li>{format!("{}", info.nickname)}</li>
                    }).collect::<Html>()
                }
                </ul>
            </div>
        }
    }
}
