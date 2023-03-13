use std::{ops::Deref, rc::Rc};

use gitcg_sim::{
    deck::*,
    game_tree_search::{*, Game},
    mcts::*,
    ids::*,
    types::{by_player::*, game_state::*, input::*, nondet::*, dice_counter::*, enums::*},
};
use gitcg_sim::{rand::prelude::*, smallvec::smallvec};
use yew::prelude::*;

use crate::{actions_list::*, search::*};
use crate::components::*;

const NDH: StandardNondetHandler = StandardNondetHandler();

pub type G = GameStateWrapper<'static, StandardNondetHandlerState>;

pub enum AppAction {
    PerformAction(Input),
    RunSearch(PlayerId),
    DispatchSearch(PlayerId, SearchAction<G>),
    SetMessage(String),
}

#[derive(Clone)]
pub struct AppState {
    pub game_state: Rc<G>,
    pub search: ByPlayer<Rc<SearchState<G>>>,
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
            search: (
                Rc::new(SearchState::new(MCTS::new(config))),
                Rc::new(SearchState::new(MCTS::new(config)))
            ).into(),
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
            AppAction::PerformAction(action) => 'a: {
                let Some(player_id) = action.player() else {
                    break 'a
                };
                let mut game_state: G = self.game_state.clone().deref().clone();
                if let Err(e) = game_state.advance(action) {
                    println!("reduce: Error: {e:?}")
                } else {
                    let search = self.search[player_id].clone();
                    next.search[player_id] = search.reduce(SearchAction::Abandon);
                    next.game_state = game_state.into();
                }
            },
            AppAction::RunSearch(maximize_player) => {
                let search = self.search[maximize_player].clone();
                next.search[maximize_player] = search.reduce(SearchAction::Start {
                    maximize_player, game_state: next.game_state.clone(), time_ms_per_step: 300, steps: 5
                });
            }
            AppAction::DispatchSearch(player_id, action) => {
                let search = self.search[player_id].clone();
                next.search[player_id] = search.reduce(action);
            },
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
        let search_player = PlayerId::PlayerSecond;
        let to_move = app_state.game_state.to_move();
        let run_search = to_move == Some(search_player);
        let search = app_state.search[search_player].clone();
        let changed_check = (
            search.solution.is_some(),
            search.search_steps.as_ref().map(|s| s.steps_remaining)
        );
        let search_finished = search.search_steps.as_ref().map(|s| s.steps_remaining == 0u32).unwrap_or(false);
        let search_started = search.search_steps.is_some() && !search_finished;
        let app_state = app_state.clone();
        let search = search.clone();
        use_effect_with_deps(move |(_, _, run_search, search_started, search_finished)| {
            if !run_search {
                return
            }

            let dt_ns = search.search_steps.as_ref().map(|x| x.total_time_ms * 1_000_000).unwrap_or_default();
            if *search_finished {
                let Some(sln) = &search.solution else { return; };
                let Some(action) = sln.pv.head() else { return; };
                app_state.dispatch(AppAction::SetMessage(format!(
                    "Search finished. {} {:?}",
                    sln.counter.summary(dt_ns),
                    sln.counter
                )));
                app_state.dispatch(AppAction::PerformAction(action));
            } else if *search_started {
                app_state.dispatch(AppAction::SetMessage(format!(
                    "Search started ({} steps remaining). {}",
                    search.search_steps.as_ref().map(|x| x.steps_remaining).unwrap_or_default(),
                    search.solution.as_ref().map(|s| s.counter.summary(dt_ns)).unwrap_or_default()
                )));
                app_state.dispatch(AppAction::DispatchSearch(search_player, SearchAction::Step));
            } else {
                app_state.dispatch(AppAction::SetMessage("Search started.".to_string()));
                app_state.dispatch(AppAction::DispatchSearch(search_player, SearchAction::Step));
            }
        }, (hash, changed_check, run_search, search_started, search_finished));
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
