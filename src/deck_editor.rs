use std::{collections::HashMap, rc::Rc};

use gitcg_sim::{ids::*, deck::Decklist};
use serde::{Serialize, Deserialize};
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::{prelude::*, html::{onchange}};
use lazy_static::{lazy_static};
use gitcg_sim::enum_map::Enum;
use gloo::storage::LocalStorage;
use gloo_storage::Storage;

use crate::actions_list::CostInfo;

const RESTRICTED_CARDS: [CardId; 5] = [
    CardId::BlankCard,
    CardId::LightningStiletto,
    CardId::Rust,
    CardId::SacrificialGreatsword,
    CardId::SkywardPride,
];

lazy_static! {
    pub static ref DECK1: Decklist = Decklist {
        characters: vec![
            CharId::Xingqiu,
            CharId::Ganyu,
            CharId::Mona,
        ].into(),
        cards: vec![
            CardId::TheBestestTravelCompanion,
            CardId::TheBestestTravelCompanion,
            CardId::ChangingShifts,
            CardId::ChangingShifts,
            CardId::LeaveItToMe,
            CardId::LeaveItToMe,
            CardId::Starsigns,
            CardId::Starsigns,
            CardId::Strategize,
            CardId::Strategize,
            CardId::IHaventLostYet,
            CardId::IHaventLostYet,
            CardId::ElementalResonanceWovenWaters,
            CardId::ElementalResonanceWovenWaters,
            CardId::MondstadtHashBrown,
            CardId::MondstadtHashBrown,
            CardId::MushroomPizza,
            CardId::MushroomPizza,
            CardId::Paimon,
            CardId::Paimon,
            CardId::LiuSu,
            CardId::LiuSu,
            CardId::IronTongueTian,
            CardId::IronTongueTian,
            CardId::DawnWinery,
            CardId::DawnWinery,
            CardId::Katheryne,
            CardId::Katheryne,
            CardId::FavoniusCathedral,
            CardId::FavoniusCathedral,
        ].into(),
    };

    pub static ref DECK2: Decklist = Decklist {
        characters: vec![
            CharId::Klee,
            CharId::Mona,
            CharId::FatuiPyroAgent,
        ].into(),
        cards: vec![
            CardId::TheBestestTravelCompanion,
            CardId::TheBestestTravelCompanion,
            CardId::ChangingShifts,
            CardId::ChangingShifts,
            CardId::LeaveItToMe,
            CardId::LeaveItToMe,
            CardId::Starsigns,
            CardId::Starsigns,
            CardId::Strategize,
            CardId::Strategize,
            CardId::IHaventLostYet,
            CardId::IHaventLostYet,
            CardId::ElementalResonanceWovenFlames,
            CardId::ElementalResonanceWovenFlames,
            CardId::MondstadtHashBrown,
            CardId::MondstadtHashBrown,
            CardId::MushroomPizza,
            CardId::MushroomPizza,
            CardId::Paimon,
            CardId::Paimon,
            CardId::LiuSu,
            CardId::LiuSu,
            CardId::IronTongueTian,
            CardId::IronTongueTian,
            CardId::DawnWinery,
            CardId::DawnWinery,
            CardId::Katheryne,
            CardId::Katheryne,
            CardId::FavoniusCathedral,
            CardId::FavoniusCathedral,
        ].into(),
    };

    pub static ref DECK3: Decklist = Decklist {
        characters: vec![
            CharId::KamisatoAyaka,
            CharId::Yoimiya,
            CharId::FatuiPyroAgent,
        ].into(),
        cards: vec![
            CardId::TheBestestTravelCompanion,
            CardId::TheBestestTravelCompanion,
            CardId::ChangingShifts,
            CardId::ChangingShifts,
            CardId::LeaveItToMe,
            CardId::LeaveItToMe,
            CardId::Starsigns,
            CardId::Starsigns,
            CardId::Strategize,
            CardId::Strategize,
            CardId::IHaventLostYet,
            CardId::IHaventLostYet,
            CardId::ElementalResonanceWovenFlames,
            CardId::ElementalResonanceWovenFlames,
            CardId::MondstadtHashBrown,
            CardId::MondstadtHashBrown,
            CardId::MushroomPizza,
            CardId::MushroomPizza,
            CardId::Paimon,
            CardId::Paimon,
            CardId::LiuSu,
            CardId::LiuSu,
            CardId::IronTongueTian,
            CardId::IronTongueTian,
            CardId::DawnWinery,
            CardId::DawnWinery,
            CardId::Katheryne,
            CardId::Katheryne,
            CardId::FavoniusCathedral,
            CardId::FavoniusCathedral,
        ].into(),
    };

    pub static ref DECK4: Decklist = Decklist {
        characters: vec![
            CharId::KujouSara,
            CharId::JadeplumeTerrorshroom,
            CharId::Keqing,
        ].into(),
        cards: vec![
            CardId::TheBestestTravelCompanion,
            CardId::TheBestestTravelCompanion,
            CardId::ChangingShifts,
            CardId::ChangingShifts,
            CardId::LeaveItToMe,
            CardId::LeaveItToMe,
            CardId::Starsigns,
            CardId::Starsigns,
            CardId::Strategize,
            CardId::Strategize,
            CardId::IHaventLostYet,
            CardId::IHaventLostYet,
            CardId::ElementalResonanceWovenThunder,
            CardId::ElementalResonanceWovenThunder,
            CardId::MondstadtHashBrown,
            CardId::MondstadtHashBrown,
            CardId::MushroomPizza,
            CardId::MushroomPizza,
            CardId::Paimon,
            CardId::Paimon,
            CardId::LiuSu,
            CardId::LiuSu,
            CardId::IronTongueTian,
            CardId::IronTongueTian,
            CardId::DawnWinery,
            CardId::DawnWinery,
            CardId::Katheryne,
            CardId::Katheryne,
            CardId::FavoniusCathedral,
            CardId::FavoniusCathedral,
        ].into(),
    };

    pub static ref CARDS: HashMap<&'static str, CardId> = {
        let n = <CardId as Enum>::LENGTH;
        let mut cards = HashMap::with_capacity(n);
        for i in 0..n {
            let card_id = CardId::from_usize(i);
            if RESTRICTED_CARDS.contains(&card_id) {
                continue;
            }
            cards.insert(card_id.get_card().name, card_id);
        }
        cards
    };

    pub static ref CARDS_LIST: Vec<(&'static str, CardId)> = {
        let mut v: Vec<_> = CARDS.iter().map(|(&a, &b)| (a, b)).collect();
        v.sort_by_key(|(card_name, card_id)| (card_id.get_card().card_type, *card_name));
        v
    };

    pub static ref CHARS: HashMap<&'static str, CharId> = {
        let n = <CharId as Enum>::LENGTH;
        let mut chars = HashMap::with_capacity(n);
        for i in 0..n {
            let char_id = CharId::from_usize(i);
            chars.insert(char_id.get_char_card().name, char_id);
        }
        chars
    };

    pub static ref CHARS_LIST: Vec<(&'static str, CharId)> = {
        let mut v: Vec<_> = CHARS.iter().map(|(&a, &b)| (a, b)).collect();
        v.sort_by_key(|(name, _)| (*name));
        v
    };
}

pub enum DeckEditorAction {
    AddChar(CharId),
    RemoveChar(CharId),
    AddCard(CardId),
    RemoveCard(CardId),
    UpdateName(String),
    LoadDeck(String),
    Save,
}

#[derive(Default, Clone, PartialEq, Eq)]
pub struct DeckEditorState {
    pub chars: Vec<CharId>,
    pub cards: Vec<CardId>,
    pub name: String,
}

const MAX_CHARS: usize = 4;
const MIN_CARDS: usize = 0;
const MAX_CARDS: usize = 40;

impl DeckEditorState {
    pub fn to_decklist(self: Rc<Self>) -> Decklist {
        Decklist::new(self.chars.clone().into(), self.cards.clone().into())
    }
}

const KEY: &str = "gicg_sim_web_decks";

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decks {
    pub decks: HashMap<String, Decklist>
}

impl Default for Decks {
    fn default() -> Self {
        let mut decks = HashMap::with_capacity(4);
        decks.insert("Deck 1".to_string(), DECK1.clone());
        decks.insert("Deck 2".to_string(), DECK2.clone());
        decks.insert("Deck 3".to_string(), DECK3.clone());
        decks.insert("Deck 4".to_string(), DECK4.clone());
        Self { decks }
    }
}

impl Reducible for DeckEditorState {
    type Action = DeckEditorAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next: DeckEditorState = DeckEditorState::clone(self.as_ref());
        match action {
            DeckEditorAction::AddChar(char_id) => {
                if next.chars.len() < MAX_CHARS && !next.chars.contains(&char_id) {
                    next.chars.push(char_id);
                }
            },
            DeckEditorAction::RemoveChar(char_id) => {
                if !next.chars.is_empty() {
                    next.chars.retain(|&c| c != char_id);
                }
            },
            DeckEditorAction::AddCard(card_id) => {
                if next.cards.len() < MAX_CARDS {
                    let count = next.cards.iter().filter(|&&c| c == card_id).count();
                    if count < 2 {
                        next.cards.push(card_id);
                        next.cards.sort_by_key(|c| c.get_card().name);
                    }
                }
            },
            DeckEditorAction::RemoveCard(card_id) => {
                if next.cards.len() > MIN_CARDS {
                    if let Some((i, _)) = next.cards.iter().enumerate().find(|(_, &c)| c == card_id) {
                        next.cards.remove(i);
                    }
                }
            },
            DeckEditorAction::UpdateName(name) => {
                // let mut decks: Decks = LocalStorage::get(KEY).unwrap_or_default();
                // decks.decks.insert(name.clone(), self.to_decklist());
                // if let Err(e) = LocalStorage::set(&name, decks) {
                //     gloo::console::error!(format!("Can't save into localStorage: {:#?}", e));
                // }
                next.name = name;
            },
            DeckEditorAction::LoadDeck(name) => {
                let decks: Decks = LocalStorage::get(KEY).unwrap_or_default();
                if let Some(deck) = decks.decks.get(&name) {
                    next.chars = deck.characters.to_vec();
                    next.cards = deck.cards.to_vec();
                    next.name = name;
                }
            },
            DeckEditorAction::Save => {
                let name = next.name.clone();
                let mut decks: Decks = LocalStorage::get(KEY).unwrap_or_default();
                decks.decks.insert(name.clone(), self.to_decklist());
                if let Err(e) = LocalStorage::set(KEY, decks) {
                    gloo::console::error!(format!("Can't save into localStorage: {:#?}", e));
                }
            },
        }
        Rc::new(next)
    }
}

#[derive(Properties, PartialEq)]
pub struct DeckEditorProps {
}

#[function_component(DeckEditor)]
pub fn deck_editor(_: &DeckEditorProps) -> Html {
    let state = use_reducer_eq(DeckEditorState::default);
    let chars = state.chars.clone();
    let cards = state.cards.clone();
    let save_deck = use_callback({
        let state = state.clone();
        move |_, ()| {
            state.dispatch(DeckEditorAction::Save)
        }
    }, ());
    let onchange = use_callback({
        let state = state.clone();
        move |e: onchange::Event, ()| {
            let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) else {
                return
            };
            state.dispatch(DeckEditorAction::UpdateName(input.value()))
        }
    }, ());
    let (deck_names, decks): (Vec<String>, Decks) = {
        let decks = LocalStorage::get::<Decks>(KEY).unwrap_or_default();
        let mut deck_names: Vec<_> = decks.decks.keys().cloned().collect();
        deck_names.sort();
        (deck_names, decks)
    };
    let load_deck = use_callback({
        let state = state.clone();
        move |e: onchange::Event, ()| {
            let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) else {
                return
            };
            state.dispatch(DeckEditorAction::LoadDeck(input.value()))
        }
    }, ());
    html! {
        <div class="deck-editor">
            <h1>{"Deck Editor"}</h1>
            <div class="deck-editor-form">
                <label for="deck-select">
                    {"Deck: "}
                    <select name="decks" value={state.name.clone()} onchange={load_deck}>
                        {for deck_names.iter().map(|name| {
                            let deck_summary = if let Some(decklist) = decks.decks.get(name) {
                                let n = decklist.cards.len();
                                let char_names: Vec<_> = decklist.characters.iter().map(|c| c.get_char_card().name).collect();
                                let chars = char_names.join(", ");
                                format!("{name} | {chars} | {n}")
                            } else {
                                name.to_string()
                            };
                            html! { <option value={name.clone()}>{deck_summary}</option> }
                        })}
                    </select>
                </label>
                <label for="deck-name">
                    {"Deck Name: "}
                    <input id="deck-name" type="text" value={state.name.clone()} {onchange} /><br />
                </label>
                <div class="save-deck">
                    <button onclick={save_deck}>{"Save Deck"}</button>
                </div>
            </div>
            <div class="deck-editor-body">
                <div>
                    <table>
                        <thead>
                            <tr>
                                <th>{"Name"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {for chars.iter().map({
                                let state = state.clone();
                                move |&char_id| {
                                    let state = state.clone();
                                    let onclick = Callback::from(move |_| {
                                        state.dispatch(DeckEditorAction::RemoveChar(char_id))
                                    });
                                    html! {
                                        <tr>
                                            <td>
                                                <button title="Remove character from deck" {onclick}>
                                                    {char_id.get_char_card().name}
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }
                            })}
                            {if chars.is_empty() {
                                Some(html! {
                                    <tr>
                                        <td colspan="3"><em>{"(No characters)"}</em></td>
                                    </tr>
                                })
                            } else { None }}
                            <tr><td colspan="3"><hr /></td></tr>
                            {for cards.iter().map({
                                let state = state.clone();
                                move |&card_id| {
                                    let state = state.clone();
                                    let onclick = Callback::from(move |_| {
                                        state.dispatch(DeckEditorAction::RemoveCard(card_id))
                                    });
                                    html! {
                                        <tr>
                                            <td>
                                                <button title="Remove card from deck" {onclick}>
                                                    {card_id.get_card().name}
                                                </button>
                                            </td>
                                        </tr>
                                    }
                                }
                            })}
                            {if cards.is_empty() {
                                Some(html! {
                                    <tr>
                                        <td colspan="3"><em>{"(No cards)"}</em></td>
                                    </tr>
                                })
                            } else { None }}
                        </tbody>
                    </table>
                </div>
                <div class="add-remove">
                    {"\u{2190}"}<br />
                    {"Add/Remove"}<br />
                    {"\u{2192}"}
                </div>
                <div>
                    <table>
                        <thead>
                            <tr>
                                <th>{"Name"}</th>
                                <th>{"Type"}</th>
                                <th>{"Cost"}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {for CHARS_LIST.iter().map({
                                let state = state.clone();
                                move |(name, char_id)| {
                                    let char = char_id.get_char_card();
                                    let state = state.clone();
                                    let onclick = Callback::from(move |_| {
                                        state.dispatch(DeckEditorAction::AddChar(*char_id))
                                    });
                                    html! {
                                        <tr>
                                            <td>
                                                <button title="Add characer to deck" {onclick}>
                                                    {name}
                                                </button>
                                            </td>
                                            <td>
                                                {format!("{:?}, {:?}", char.elem, char.faction)}
                                            </td>
                                            <td></td>
                                        </tr>
                                    }
                                }
                            })}
                            <tr><td colspan="3"><hr /></td></tr>
                            {for CARDS_LIST.iter().map({
                                let state = state.clone();
                                move |(name, card_id)| {
                                    let card = card_id.get_card();
                                    let state = state.clone();
                                    let onclick = Callback::from(move |_| {
                                        state.dispatch(DeckEditorAction::AddCard(*card_id))
                                    });
                                    html! {
                                        <tr>
                                            <td>
                                                <button title="Add card to deck" {onclick}>
                                                    {name}
                                                </button>
                                            </td>
                                            <td>
                                                {format!("{:?}", card.card_type)}
                                            </td>
                                            <td>
                                                <CostInfo cost={card.cost} />
                                            </td>
                                        </tr>
                                    }
                                }
                            })}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
}
