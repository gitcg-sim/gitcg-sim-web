use std::{ops::Deref, rc::Rc, cell::{RefMut, RefCell}};

use gitcg_sim::{
    deck::*,
    game_tree_search::{*, Game},
    ids::*,
    types::{game_state::*, input::*, nondet::*, dice_counter::*, enums::*},
};
use gitcg_sim::{rand::prelude::*, smallvec::smallvec};
use yew::prelude::*;
use yew_agent::{Bridged};

use crate::{actions_list::*, search::*};
use crate::components::*;

pub type G = GameStateWrapper<StandardNondetHandlerState>;

pub enum AppAction {
    PerformAction(Input),
    SetMessage(String),
}

#[derive(Clone)]
pub struct AppState {
    pub game_state: Rc<G>,
    pub message: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            game_state: Rc::new(default_game_state()),
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
                if action.player().is_none() {
                    break 'a
                }
                let mut game_state: G = self.game_state.clone().deref().clone();
                if let Err(e) = game_state.advance(action) {
                    println!("reduce: Error: {e:?}")
                } else {
                    next.game_state = game_state.into();
                }
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

fn default_game_state() -> G {
    let rand1 = SmallRng::seed_from_u64(100);
    let decklist1 = Decklist::new(
        smallvec![CharId::Yoimiya, CharId::Xingqiu, CharId::KamisatoAyaka],
        sample_deck(),
    );
    let decklist2 = Decklist::new(
        smallvec![CharId::Mona, CharId::Fischl, CharId::Collei],
        sample_deck(),
    );
    new_standard_game(&decklist1, &decklist2, rand1)
}

#[function_component(App)]
pub fn app() -> Html {
    let app = use_reducer(AppState::default);
    let player_to_move = app.game_state.to_move();
    let hash = app.game_state.zobrist_hash();
    let search_callback: Rc<RefCell<Option<Callback<SearchReturn>>>> = use_mut_ref(|| None);

    let handler = {
        //let app = app.clone();
        let search_callback = search_callback.clone();
        use_mut_ref(move || {
            let handler = {
                SearchWorker::bridge(Rc::new(move |msg| {
                    if let Ok(c) = search_callback.try_borrow() {
                        c.iter().for_each(|c| {
                            c.emit(msg.clone())
                        })
                    }
                }))
            };
            handler
        })
    };

    {
        let app = app.clone();
        let handler = handler.clone();
        *search_callback.try_borrow_mut().unwrap() = Some(Callback::from(move |msg| {
            match msg {
                SearchReturn(false, Some(res), total_time_ns) => {
                    gloo::console::log!("Step");
                    app.dispatch(AppAction::SetMessage(format!(
                        "Step {}, {} states visited",
                        res.counter.summary(total_time_ns), res.counter.states_visited)));
                    handler.try_borrow_mut().unwrap().send(SearchAction::Step);
                },
                SearchReturn(true, Some(res), total_time_ns) => {
                    gloo::console::log!("Finish");
                    app.dispatch(AppAction::SetMessage(format!(
                        "Finished {}, {} states visited",
                        res.counter.summary(total_time_ns), res.counter.states_visited)));
                    let Some(head) = res.pv.head() else { return };
                    app.dispatch(AppAction::PerformAction(head));
                },
                SearchReturn(_, None, _) => {
                    gloo::console::log!("Empty");
                    handler.try_borrow_mut().unwrap().send(SearchAction::Step);
                },
            }
        }));
    }

    {
        let handler = handler.clone();
        let app = app.clone();
        use_effect_with_deps(move |(_, player_to_move)| {
            if let Some(player_id) = *player_to_move {
                if player_id == PlayerId::PlayerSecond {
                    let mut r: RefMut<Box<_>> = handler.as_ref().borrow_mut();
                    r.send(SearchAction::Start {
                        maximize_player: PlayerId::PlayerSecond,
                        game_state: app.game_state.clone(),
                        steps: 3,
                    });
                }
            }
        }, (hash, player_to_move));
    }

    let active_player = app.game_state.game_state.get_active_player();
    let dice = active_player.map(|p| p.dice);
    html! {
        <main>
            <h1>{ "GITCGSim Web" }</h1>
            <div class="col">
                <Board game_state={app.game_state.clone()} hash={app.game_state.zobrist_hash()} />
                <div>
                    <h2>{"Dice"}</h2>
                    {dice.map(|dice| html! { <DiceList {dice} /> })}
                    <h2>{"Actions"}</h2>
                    <ActionsList app={app.clone()} />
                </div>
            </div>
            <div class="col">
                <pre class="codebox">
                    {&app.message}
                </pre>
            </div>
            <div class="col">
                <pre class="codebox">
                    {format!("{:#?}", &app.game_state)}
                </pre>
                <pre class="codebox">
                    {format!("{:#?}", &app.game_state.nd.state)}
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
