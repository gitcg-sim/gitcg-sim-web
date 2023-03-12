use std::rc::Rc;

use gitcg_sim::{
    ids::*,
    types::{card_defs::Status, enums::{Element, EquipSlot}, game_state::*, nondet::StandardNondetHandlerState}, game_tree_search::GameStateWrapper,
};
use yew::prelude::*;

#[derive(Properties)]
pub struct BoardProps {
    pub game_state: Rc<GameStateWrapper<'static, StandardNondetHandlerState>>,
    pub hash: u64
}

impl PartialEq for BoardProps {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[function_component(Board)]
pub fn board(props: &BoardProps) -> Html {
    let game_state = &props.game_state.game_state;
    let count_p1 = props.game_state.nd.state.decks.0.count;
    let count_p2 = props.game_state.nd.state.decks.1.count;
    let hash = game_state.zobrist_hash();
    html! {
        <div class="board">
            <h2>{"Board"}</h2>
            <PlayerPart player_state={game_state.get_player(PlayerId::PlayerSecond).clone()} player_id={PlayerId::PlayerSecond} {hash} />
            <PlayerDeck player_id={PlayerId::PlayerSecond} deck_count={count_p2} dice_count={game_state.get_player(PlayerId::PlayerSecond).dice.total()} />
            <div class="divider" />
            <PlayerPart player_state={game_state.get_player(PlayerId::PlayerFirst).clone()} player_id={PlayerId::PlayerFirst} {hash} />
            <PlayerDeck player_id={PlayerId::PlayerFirst} deck_count={count_p1} dice_count={game_state.get_player(PlayerId::PlayerFirst).dice.total()} />
        </div>
    }
}

#[derive(Properties)]
pub struct PlayerPartProps {
    pub player_state: PlayerState,
    pub player_id: PlayerId,
    pub hash: u64,
}

impl PartialEq for PlayerPartProps {
    fn eq(&self, other: &Self) -> bool {
        self.player_id == other.player_id && self.hash == other.hash
    }
}

#[derive(Properties, PartialEq)]
pub struct PlayerDeckProps {
    pub player_id: PlayerId,
    pub deck_count: u8,
    pub dice_count: u8
}

#[function_component(PlayerDeck)]
pub fn player_deck(props: &PlayerDeckProps) -> Html {
    html! {
        <div class={classes!("player-deck", props.player_id.to_string())}>
            <h3>{"Player Deck"}</h3>
            <div class="player-deck-card" title={format!("Cards in deck for {}", props.player_id)}>
                {props.deck_count}
            </div>
            <div class="player-deck-dice" title={format!("Dice count for {}", props.player_id)}>
                {props.dice_count}
            </div>
        </div>
    }
}

#[function_component(PlayerPart)]
pub fn player_part(props: &PlayerPartProps) -> Html {
    let PlayerPartProps { player_state, hash, .. } = props;
    let chars = &player_state.char_states;
    let active = player_state.active_char_index;
    let summons = player_state.status_collection.summon_statuses_vec();
    let supports = player_state.status_collection.support_statuses_vec();
    let hidden = props.player_id == PlayerId::PlayerSecond;
    html! {
        <div class={classes!("player-part", props.player_id.to_string())}>
            <h3>{format!("Player {}", props.player_id)}</h3>
            <div class="player-supports">
                <h4>{"Supports"}</h4>
                {for supports.iter().copied().map(|&support| html! {
                    <Support {support} />
                })}
            </div>
            <div class="player-characters">
                <h4>{"Characters"}</h4>
                {for chars.iter().enumerate().map(|(i, c)| {
                    let is_active = (i as u8) == active;
                    let equip_statuses: Vec<(EquipSlot, StatusId, AppliedEffectState)> = player_state
                        .status_collection
                        .equipment_statuses_vec(i as u8)
                        .iter()
                        .copied()
                        .map(|(slot, status, state)| (slot, status, *state))
                        .collect();
                    let char_statuses: Vec<StatusEntry> = player_state.status_collection.character_statuses_vec(i as u8)
                        .iter().copied().copied().collect();
                    let team_statuses: Vec<StatusEntry> = if is_active {
                        player_state.status_collection.team_statuses_vec()
                            .iter().copied().copied().collect()
                    } else { vec![] };
                    html! {
                        <Character
                            char_state={c.clone()}
                            {is_active}
                            {equip_statuses}
                            {char_statuses}
                            {team_statuses}
                            {hash}
                        />
                    }
                })}
            </div>
            <div class="player-summons">
                <h4>{"Summons"}</h4>
                {for summons.iter().copied().map(|&summon| html! {
                    <Summon {summon} />
                })}
            </div>
            <div class="player-hand">
                {for player_state.hand.iter().copied().map(|card_id| html! {
                    <Card {card_id} {hidden} />
                })}
            </div>
        </div>
    }
}

#[derive(Properties)]
pub struct CharacterProps {
    pub is_active: bool,
    pub char_state: CharState,
    pub equip_statuses: Vec<(EquipSlot, StatusId, AppliedEffectState)>,
    pub char_statuses: Vec<StatusEntry>,
    pub team_statuses: Vec<StatusEntry>,
    pub hash: u64,
}

impl PartialEq for CharacterProps {
    fn eq(&self, other: &Self) -> bool {
        self.is_active == other.is_active && self.hash == other.hash
    }
}

#[function_component(Character)]
pub fn char_part(props: &CharacterProps) -> Html {
    let char_state = &props.char_state;
    let char_card = char_state.char_id.get_char_card();
    let is_dead = char_state.get_hp() == 0;
    let status_line = |class: &'static str, status: &'static Status, state: AppliedEffectState| {
        html! {
            <li class={class}>
                {status.name}
                {" "}
                <StatusInfo {status} {state} compact={true} />
            </li>
        }
    };
    html! {
        <div class={classes!(
            "char-card",
            if props.is_active { Some("is-active") } else { None },
            if is_dead { Some("is-dead") } else { None })
        } title="Character Card">
            <h5>{char_card.name}</h5>
            <ul>
                <li class="char-elements">
                    {for char_state.applied.iter().map(|element| html!{
                        <Elem {element} />
                    })}
                </li>
                <li>{format!("HP: {}/{}", char_state.get_hp(), char_card.max_health)}</li>
                <li>{format!("Energy: {}/{}", char_state.get_energy(), char_card.max_energy)}</li>
                {
                    if !is_dead {
                        html! {
                            <li>
                                <div class="char-statuses">
                                    <h6>{"Statuses:"}</h6>
                                    <ul>
                                        {for props.equip_statuses.iter().copied().map(|(slot, status_id, state)| {
                                            let status = status_id.get_status();
                                            html! {
                                                <li class={format!("status-equip equip-slot-{slot:?}")}>
                                                    {format!("{:?}: ", slot)}
                                                    {status.name}
                                                    {" "}
                                                    <StatusInfo {status} {state} compact={true} />
                                                </li>
                                            }
                                        })}
                                        {for props.char_statuses.iter().map(|s| {
                                            if let Some(status) = s.status_id().map(|s| s.get_status()) {
                                                status_line("char-status", status, s.state)
                                            } else {
                                                html! { }
                                            }
                                        })}
                                    </ul>
                                    <hr />
                                    <ul>
                                        {for props.team_statuses.iter().map(|s| {
                                            if let Some(status) = s.status_id().map(|s| s.get_status()) {
                                                status_line("team-status", status, s.state)
                                            } else {
                                                html! { }
                                            }
                                        })}
                                    </ul>
                                </div>
                            </li>
                        }
                    } else {
                        html! {}
                    }
                }
            </ul>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SummonProps {
    pub summon: StatusEntry,
}

#[function_component(Summon)]
fn summon_part(props: &SummonProps) -> Html {
    let summon = &props.summon;
    let Some(summon_id) = summon.summon_id() else {
        return html! { <div class={"summon-not-found"} /> }
    };
    let status = summon_id.get_status();
    html! {
        <div class="summon">
            <h5>{status.name}</h5>
            <StatusInfo {status} state={summon.state} />
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct SupportProps {
    pub support: StatusEntry,
}

#[function_component(Support)]
fn support_part(props: &SupportProps) -> Html {
    let support = &props.support;
    let Some(support_id) = support.support_id() else {
        return html! { <div class={"support-not-found"} /> }
    };
    let status = support_id.get_status();
    html! {
        <div class="support">
            <h5>{status.name}</h5>
            <StatusInfo {status} state={support.state} />
        </div>
    }
}

#[derive(Properties)]
pub struct StatusInfoProps {
    pub status: &'static Status,
    pub state: AppliedEffectState,
    #[prop_or_default]
    pub compact: bool,
}

impl PartialEq for StatusInfoProps {
    fn eq(&self, other: &Self) -> bool {
        self.status.name == other.status.name && self.state == other.state
    }
}

#[function_component(StatusInfo)]
fn status_info(StatusInfoProps { status, state, compact }: &StatusInfoProps) -> Html {
    if *compact {
        if status.usages.is_some() {
            html!{ {format!("({})", state.get_usages())} }
        } else if status.duration_rounds.is_some() {
            html!{ {format!("({})", state.get_duration())} }
        } else {
            html!{}
        }
    } else {
        html! {
            <div>
                {
                    if status.usages.is_some() {
                        html!{ <span>{format!("Usages: {}", state.get_usages())}</span> }
                    } else if status.duration_rounds.is_some() {
                        html!{ <span>{format!("Duration: {}", state.get_duration())}</span> }
                    } else {
                        html!{}
                    }
                }
            </div>
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct CardProps {
    pub card_id: CardId,
    #[prop_or(false)]
    pub hidden: bool,
}

#[function_component(Card)]
fn card(CardProps { card_id, hidden }: &CardProps) -> Html {
    let card = card_id.get_card();
    html! {
        <span class="card" title="Card">
            {if *hidden { "" } else { card.name }}
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct ElementProps {
    pub element: Element,
}

#[function_component(Elem)]
fn element(props: &ElementProps) -> Html {
    let e = props.element;
    html! {
        <span class={format!("elem-{}", e.get_name())}>
            {e.get_name()}
        </span>
    }
}
