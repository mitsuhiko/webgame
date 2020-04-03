use std::rc::Rc;

use im_rc::Vector;
use web_sys::Element;
use yew::{html, Component, ComponentLink, Html, NodeRef, Properties, ShouldRender};

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
    link: ComponentLink<ChatBox>,
    log_ref: NodeRef,
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

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        ChatBox {
            log: props.log,
            link,
            log_ref: NodeRef::default(),
        }
    }

    fn update(&mut self, _: Self::Message) -> ShouldRender {
        if let Some(div) = self.log_ref.cast::<Element>() {
            div.set_scroll_top(div.scroll_height());
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        if self.log != props.log {
            self.log = props.log;
            self.link.send_message(());
            true
        } else {
            false
        }
    }

    fn view(&self) -> Html {
        html! {
            <div class="chat box">
                <h2>{"Chat"}</h2>
                <ul id="chat-log" ref=self.log_ref.clone()>
                {
                    for self.log.iter().map(|item| html! {
                        <li>{item.render()}</li>
                    })
                }
                </ul>
            </div>
        }
    }
}
