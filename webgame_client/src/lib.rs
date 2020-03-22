#![recursion_limit = "512"]

mod app;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    web_logger::init();
    yew::start_app::<crate::app::Model>();
    Ok(())
}