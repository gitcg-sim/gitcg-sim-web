mod search;

mod components;

mod actions_list;

mod app;

use app::App;

fn main() {
    yew::Renderer::<App>::new().render();
}
