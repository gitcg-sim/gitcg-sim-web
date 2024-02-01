use std::{
    borrow::Borrow,
    cell::{RefCell, RefMut},
    ops::Deref,
    rc::Rc,
};

use gitcg_sim::{
    deck::sample_deck,
    prelude::{tcg_model::Dice, *},
    rand::prelude::*,
    smallvec::smallvec,
};
use gloo_storage::{LocalStorage, Storage};
use yew::prelude::*;
use yew_agent::Bridged;

use crate::components::*;
use crate::{
    actions_list::*,
    deck_editor::{DeckEditor, DeckSelector, Decks},
    search::*,
};

pub type G = GameStateWrapper<StandardNondetHandlerState>;

pub enum AppAction {
    PerformAction(Input),
    SetMessage(String),
    SetGameState(Rc<G>),
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
            AppAction::SetGameState(game_state) => {
                next.game_state = game_state;
            }
            AppAction::SetMessage(message) => {
                next.message = message;
            }
            AppAction::PerformAction(action) => 'a: {
                if action.player().is_none() {
                    break 'a;
                }
                let mut game_state: G = self.game_state.clone().deref().clone();
                if let Err(e) = game_state.advance(action) {
                    println!("reduce: Error: {e:?}")
                } else {
                    next.game_state = game_state.into();
                }
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

const RANDOM_SEED_KEY: &str = "random_seed";
const SEARCH_STEPS_KEY: &str = "search_steps";

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
                        c.iter().for_each(|c| c.emit(msg.clone()))
                    }
                }))
            };
            handler
        })
    };

    {
        let app = app.clone();
        let handler = handler.clone();
        *search_callback.try_borrow_mut().unwrap() = Some(Callback::from(move |msg| match msg {
            SearchReturn(false, Some(res), total_time_ns) => {
                app.dispatch(AppAction::SetMessage(format!(
                    "Step {}, {} states visited",
                    res.counter.summary(total_time_ns),
                    res.counter.states_visited
                )));
                handler.try_borrow_mut().unwrap().send(SearchAction::Step);
            }
            SearchReturn(true, Some(res), total_time_ns) => {
                gloo::console::log!("Finish");
                app.dispatch(AppAction::SetMessage(format!(
                    "Finished {}, {} states visited, Best Move = {}",
                    res.counter.summary(total_time_ns),
                    res.counter.states_visited,
                    res.pv
                        .head()
                        .map(|a| describe_action(&app.game_state, a))
                        .unwrap_or_default()
                )));
                let Some(head) = res.pv.head() else { return };
                app.dispatch(AppAction::PerformAction(head));
            }
            SearchReturn(_, None, _) => {
                handler.try_borrow_mut().unwrap().send(SearchAction::Step);
            }
        }));
    }

    {
        let handler = handler.clone();
        let app = app.clone();
        use_effect_with_deps(
            move |(_, player_to_move)| {
                if let Some(player_id) = *player_to_move {
                    if player_id == PlayerId::PlayerSecond {
                        let mut r: RefMut<Box<_>> = handler.as_ref().borrow_mut();
                        let mut gsr = app.game_state.clone();
                        {
                            let game_state = Rc::make_mut(&mut gsr);
                            game_state.hide_private_information(PlayerId::PlayerFirst);
                        }
                        r.send(SearchAction::Start {
                            maximize_player: PlayerId::PlayerSecond,
                            game_state: gsr,
                            steps: LocalStorage::get(SEARCH_STEPS_KEY).unwrap_or(5),
                        });
                        app.dispatch(AppAction::SetMessage("Searching...".to_string()));
                    }
                }
            },
            (hash, player_to_move),
        );
    }

    let on_start = use_callback(
        move |r: Rc<(Decklist, Decklist)>, app| {
            let (decklist1, decklist2) = r.as_ref();
            let rng = SmallRng::seed_from_u64(LocalStorage::get(RANDOM_SEED_KEY).unwrap_or(100));
            app.dispatch(AppAction::SetGameState(
                new_standard_game(decklist1, decklist2, rng).into(),
            ));
        },
        app.clone(),
    );

    let active_player = app.game_state.game_state.get_active_player();
    let to_move = app.game_state.to_move();
    let dice = active_player.map(|p| p.dice);
    html! {
        <main>
            <h1>{ "GITCGSim Web" }</h1>
            <div class="col">
                <Board game_state={app.game_state.clone()} hash={app.game_state.zobrist_hash()} />
                <div class="moves-list">
                    <h2>{"Dice"}</h2>
                    {if to_move == Some(PlayerId::PlayerFirst) {
                        html! { for dice.map(|dice| html! { <DiceList {dice} /> }) }
                    } else {
                        html! { " - " }
                    }}
                    <h2>{"Actions"}</h2>
                    {if to_move == Some(PlayerId::PlayerFirst) {
                        html! { <ActionsList app={app.clone()} /> }
                    } else {
                        html! { " - " }
                    }}
                </div>
            </div>
            <div>
                <pre class="codebox">
                    {&app.message}
                </pre>
            </div>
            <hr />
            <div class="col">
                <StartGameForm {on_start} />
                <DeckEditor />
            </div>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct DiceListProps {
    dice: DiceCounter,
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

#[derive(Properties, PartialEq)]
struct StartGameFormProps {
    on_start: Callback<Rc<(Decklist, Decklist)>>,
}

#[function_component(StartGameForm)]
fn start_game_form(props: &StartGameFormProps) -> Html {
    let on_start = props.on_start.clone();
    let deck_p1 = use_state(|| "Deck 1".to_string());
    let deck_p2 = use_state(|| "Deck 2".to_string());
    let onclick = use_callback(
        move |_, (deck_p1, deck_p2)| {
            let (d1, d2): (&String, &String) = (deck_p1, deck_p2);
            let decks = &Decks::get_from_storage().decks;
            let t = (decks.get(d1), decks.get(d2));
            let (Some(decklist1), Some(decklist2)) = t else {
                return;
            };
            on_start.emit(Rc::new((decklist1.clone(), decklist2.clone())))
        },
        (deck_p1.clone(), deck_p2.clone()),
    );

    html! {
        <div>
            <h2>{"Start Game"}</h2>
            <div>
                <DeckSelector
                    title={"Player 1: "}
                    id="deck-p1"
                    selected={String::clone(deck_p1.borrow())}
                    on_select={Callback::from({
                        let deck_p1 = deck_p1.clone();
                        move |v| deck_p1.set(v)
                    })}
                />
            </div>
            <div>
                <DeckSelector
                    title={"Player 2: "}
                    id="deck-p2"
                    selected={String::clone(deck_p2.borrow())}
                    on_select={Callback::from({
                        let deck_p2 = deck_p2.clone();
                        move |v| deck_p2.set(v)
                    })}
                />
            </div>
            <div>
                <button {onclick}>{"Start"}</button>
            </div>
        </div>
    }
}

pub fn describe_action_with_player(game_state: &G, action: Input) -> String {
    format!(
        "{}{}",
        action
            .player()
            .map(|p| format!("{p}: "))
            .unwrap_or_default(),
        describe_action(game_state, action)
    )
}

pub fn describe_action(game_state: &G, action: Input) -> String {
    let card_name = |card_id: CardId| card_id.get_card().name;
    let char_name = |char_id: CharId| char_id.get_char_card().name;
    let Input::FromPlayer(player_id, act) = action else {
        return "\u{2205}".to_string();
    };
    let get_char = |i: u8| &game_state.game_state.get_player(player_id).char_states[i];
    match act {
        PlayerAction::EndRound => "End Round".to_string(),
        PlayerAction::PlayCard(card_id, target) => {
            let target_part = target.map(|t| match t {
                CardSelection::OwnCharacter(i) => char_name(get_char(i).char_id).to_string(),
                CardSelection::OwnSummon(s) => format!("Own Summon({})", s.get_status().name),
                CardSelection::OpponentSummon(s) => format!("Opp. Summon({})", s.get_status().name),
            });
            if let Some(t) = target_part {
                format!("Card({}, {t})", card_name(card_id))
            } else {
                format!("Card({})", card_name(card_id))
            }
        }
        PlayerAction::ElementalTuning(card_id) => format!("ET({})", card_name(card_id)),
        PlayerAction::CastSkill(skill_id) => format!("Cast({})", skill_id.get_skill().name),
        PlayerAction::SwitchCharacter(i) | PlayerAction::PostDeathSwitch(i) => {
            format!("Switch({})", char_name(get_char(i).char_id))
        }
    }
}
