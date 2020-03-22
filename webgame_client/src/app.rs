use yew::{Component, ComponentLink, ShouldRender, html, Html};

pub struct Model;

impl Component for Model {
    type Message = ();
    type Properties = ();

    fn create(_: Self::Properties, _link: ComponentLink<Self>) -> Self {
        Model
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>{"Testing this World!"}</div>
        }
    }
}