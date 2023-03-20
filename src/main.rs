mod search;

mod components;

mod actions_list;

mod app;

mod deck_editor;

use crate::search::SearchWorker;
use app::App;
use wasm_bindgen::prelude::*;
use yew_agent::PublicWorker;

pub fn main() {
    use js_sys::{global, Reflect};
    if Reflect::has(&global(), &JsValue::from_str("window")).unwrap() {
        yew::Renderer::<App>::new().render();
    } else {
        SearchWorker::register();
    }
}
