
mod search;

mod components;

mod actions_list;

mod app;

use app::App;
use yew_agent::PublicWorker;
use wasm_bindgen::prelude::*;
use crate::search::SearchWorker;

pub fn main() {
    use js_sys::{global, Reflect};
    gloo::console::log!("main start");
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        yew::Renderer::<App>::new().render();
    } else {
        gloo::console::log!("inside worker");
        SearchWorker::register();
    }
}
