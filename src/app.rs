use std::{ops::Deref, rc::Rc, cell::RefCell, borrow::Borrow};

use gitcg_sim::{
    deck::*,
    game_tree_search::{*, Game},
    ids::*,
    types::{game_state::*, input::*, nondet::*, dice_counter::DiceCounter, enums::Dice}, mcts::{MCTSConfig, MCTS},
};
use gitcg_sim::{rand::prelude::*, smallvec::smallvec};
use yew::prelude::*;

use crate::actions_list::*;
use crate::components::*;

const NDH: StandardNondetHandler = StandardNondetHandler();

pub enum AppAction {
    PerformAction(Input),
    RunSearch(PlayerId),
    SetMessage(String),
}

pub type G = GameStateWrapper<'static, StandardNondetHandlerState>;

#[derive(Clone)]
pub struct AppState {
    pub game_state: Rc<G>,
    pub search: Rc<RefCell<MCTS<G>>>,
    pub search_solution: Option<SearchResult<G>>,
    pub message: String,
}

const TIME_LIMIT_MS: u32 = 1000;

impl Default for AppState {
    fn default() -> Self {
        let game_state = default_game_state();
        let config = MCTSConfig::new(
            TIME_LIMIT_MS,
            8.0,
            8192,
            false,
            100,
            100,
            false
        );
        Self {
            game_state: Rc::new(game_state),
            search: Rc::new(RefCell::new(MCTS::new(config))),
            search_solution: Default::default(),
            message: Default::default(),
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
        let mut next: Self = self.deref().clone();
        match action {
            AppAction::SetMessage(message) => {
                next.message = message;
            },
            AppAction::PerformAction(action) => {
                let game_state = Rc::make_mut(&mut next.game_state);
                if let Err(e) = game_state.advance(action) {
                    println!("reduce: Error: {e:?}")
                } else {
                    next.search_solution = None;
                }
            },
            AppAction::RunSearch(maximize_player) => {
                let mut search = next.search.deref().borrow_mut();
                next.search_solution = Some(search.search(next.game_state.borrow(), maximize_player));
            }
        };
        Rc::new(next)
    }
}

#[derive(Clone)]
pub struct GameStateProp(pub Rc<GameState>);

impl GameStateProp {
    pub fn new(gs: &GameState) -> Self {
        Self(Rc::new(gs.clone()))
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
    let game_state = GameState::new(&decklist1.characters, &decklist2.characters, false);
    let state = StandardNondetHandlerState::new(&decklist1, &decklist2, rand1);
    let nd = NondetProvider::new(&NDH, state);
    GameStateWrapper::new(game_state, nd)
}

#[function_component(App)]
pub fn app() -> Html {
    let app_state = use_reducer(AppState::default);
    let player_to_move = app_state.game_state.to_move();
    let hash = app_state.game_state.zobrist_hash();
    {
        let app_state = app_state.clone();
        use_effect_with_deps(move |(_, player_to_move)| {
            if let Some(player_id) = *player_to_move {
                if player_id == PlayerId::PlayerSecond {
                    app_state.dispatch(AppAction::RunSearch(player_id));
                }
            }
        }, (hash, player_to_move));
    }
    {
        let flag = app_state.search_solution.is_some();
        let app_state = app_state.clone();
        use_effect_with_deps(move |(_, _)| {
            let Some(sln) = &app_state.search_solution else { return; };
            let Some(action) = sln.pv.head() else { return; };
            app_state.dispatch(AppAction::SetMessage(sln.counter.summary((TIME_LIMIT_MS as u128) * 1_000_000)));
            app_state.dispatch(AppAction::PerformAction(action));
        }, (hash, flag));
    }


    let active_player = app_state.game_state.game_state.get_active_player();
    let dice = active_player.map(|p| p.dice);
    html! {
        <main>
            <h1>{ "GITCGSim Web" }</h1>
            <div class="col">
                <Board game_state={app_state.game_state.clone()} hash={app_state.game_state.zobrist_hash()} />
                <div>
                    <h2>{"Dice"}</h2>
                    {dice.map(|dice| html! { <DiceList {dice} /> })}
                    <h2>{"Actions"}</h2>
                    <ActionsList app_state={app_state.clone()} />
                </div>
            </div>
            <div class="col">
                <pre class="codebox">
                    {&app_state.message}
                </pre>
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
                        Dice::Omni => html! { <span class="dice dice-omni" title={"Omni"}>{"O"}</span> },
                        Dice::Elem(e) => html! {
                            <span class={format!("dice dice-elem elem-{e:?}")} title={e.get_name()}>
                                {e.get_name().chars().next()}
                            </span>
                        }
                    }
                )
            })}
        </div>
    }
}
