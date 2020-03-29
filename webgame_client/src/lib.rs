#![recursion_limit = "512"]

mod api;
mod app;
mod components;
mod utils;
mod views;

use wasm_bindgen::prelude::*;

pub(crate) use webgame_protocol as protocol;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_logger::init();
    yew::start_app::<crate::app::App>();
    Ok(())
}
