use std::{ops::Deref, rc::Rc};

use gitcg_sim::{
    deck::*,
    game_tree_search::*,
    ids::*,
    types::{game_state::*, input::*, nondet::*, dice_counter::DiceCounter, enums::Dice},
};
use gitcg_sim::{rand::prelude::*, smallvec::smallvec};
use yew::prelude::*;

use crate::actions_list::*;
use crate::components::*;

const NDH: StandardNondetHandler = StandardNondetHandler();

pub enum AppAction {
    PerformAction(Input),
}

pub struct AppState {
    pub game_state: GameStateWrapper<'static, StandardNondetHandlerState>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            game_state: default_game_state(),
        }
    }
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        self.game_state.zobrist_hash() == other.game_state.zobrist_hash()
    }
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AppAction::PerformAction(action) => {
                let mut game_state = self.game_state.clone();
                game_state.advance(action).unwrap();
                Self { game_state }.into()
            }
        }
    }
}

#[derive(Clone)]
pub struct GameStateProp(pub GameState);

impl GameStateProp {
    pub fn new(gs: &GameState) -> Self {
        Self(gs.clone())
    }
}

impl Deref for GameStateProp {
    type Target = GameState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq for GameStateProp {
    fn eq(&self, other: &Self) -> bool {
        self.0.zobrist_hash() == other.0.zobrist_hash()
    }
}

fn default_game_state() -> GameStateWrapper<'static, StandardNondetHandlerState> {
    let rand1 = SmallRng::seed_from_u64(100);
    let decklist1 = Decklist::new(
        smallvec![CharId::Yoimiya, CharId::Xingqiu, CharId::KamisatoAyaka],
        sample_deck(),
    );
    let decklist2 = Decklist::new(
        smallvec![CharId::Mona, CharId::Fischl, CharId::Collei],
        sample_deck(),
    );
    let game_state = GameState::new(&decklist1.characters, &decklist2.characters, true);
    let state = StandardNondetHandlerState::new(&decklist1, &decklist2, rand1);
    let nd = NondetProvider::new(&NDH, state);
    GameStateWrapper::new(game_state, nd)
}

#[function_component(App)]
pub fn app() -> Html {
    let app_state = use_reducer(AppState::default);
    let active_player = app_state.game_state.game_state.get_active_player();
    let dice = active_player.map(|p| p.dice);
    html! {
        <main>
            <h1>{ "GITCGSim Web" }</h1>
            <div class="col">
                <Board game_state={app_state.game_state.game_state.clone()} />
                <div>
                    <h2>{"Actions"}</h2>
                    <ActionsList app_state={app_state.clone()} />
                    <h2>{"Dice"}</h2>
                    {dice.map(|dice| html! { <DiceList {dice} /> })}
                </div>
            </div>
            <div class="col">
                <pre class="codebox">
                    {format!("{:#?}", &app_state.game_state)}
                </pre>
                <pre class="codebox">
                    {format!("{:#?}", &app_state.game_state.nd.state)}
                </pre>
            </div>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct DiceListProps {
    dice: DiceCounter
}

#[function_component(DiceList)]
fn dice_list(props: &DiceListProps) -> Html {
    html! {
        <div class="dice-list">
            {for props.dice.tally().iter().copied().flat_map(|(d, c)| {
                (0..c).into_iter().map(move |_|
                    match d {
                        Dice::Omni => html! { <span class="dice dice-omni">{"Omni"}</span> },
                        Dice::Elem(e) => html! { <span class={format!("dice dice-elem elem-{e:?}")}>{format!("{e:?}")}</span> }
                    }
                )
            })}
        </div>
    }
}