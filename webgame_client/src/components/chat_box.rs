use std::rc::Rc;

use im_rc::Vector;
use yew::{html, Component, ComponentLink, Html, Properties, ShouldRender};

#[derive(PartialEq)]
pub enum ChatLineData {
    Connected,
    Disconnected,
    Text(String),
}

#[derive(PartialEq)]
pub struct ChatLine {
    pub nickname: String,
    pub data: ChatLineData,
}

#[derive(Clone, Properties)]
pub struct Props {
    pub log: Vector<Rc<ChatLine>>,
}

pub struct ChatBox {
    log: Vector<Rc<ChatLine>>,
}

impl ChatLine {
    pub fn text(&self) -> &str {
        match self.data {
            ChatLineData::Connected => "*connected*",
            ChatLineData::Disconnected => "*disconnected*",
            ChatLineData::Text(ref x) => x.as_str(),
        }
    }

    pub fn render(&self) -> String {
        format!("<{}> {}", self.nickname, self.text())
    }
}

impl Component for ChatBox {
    type Message = ();
    type Properties = Props;

    fn create(props: Self::Properties, _link: ComponentLink<Self>) -> Self {
        ChatBox { log: props.log }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.log != props.log {
            self.log = props.log;
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="chat box">
                <h2>{"Chat"}</h2>
                <ul>
                {
                    self.log.iter().map(|item| html! {
                        <li>{item.render()}</li>
                    }).collect::<Html>()
                }
                </ul>
            </div>
        }
    }
}
