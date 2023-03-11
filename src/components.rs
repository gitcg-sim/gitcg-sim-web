use gitcg_sim::{
    ids::*,
    types::{card_defs::Status, enums::Element, game_state::*},
};
use yew::prelude::*;

#[derive(Properties)]
pub struct BoardProps {
    pub game_state: GameState,
}

impl PartialEq for BoardProps {
    fn eq(&self, other: &Self) -> bool {
        self.game_state.zobrist_hash() == other.game_state.zobrist_hash()
    }
}

#[function_component(Board)]
pub fn board(props: &BoardProps) -> Html {
    let hash = props.game_state.zobrist_hash();
    html! {
        <div class="board">
            <h2>{"Board"}</h2>
            <PlayerPart player_state={props.game_state.players.1.clone()} player_id={PlayerId::PlayerSecond} {hash} />
            <div class="divider" />
            <PlayerPart player_state={props.game_state.players.0.clone()} player_id={PlayerId::PlayerFirst} {hash} />
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

#[function_component(PlayerPart)]
pub fn player_part(props: &PlayerPartProps) -> Html {
    let player_state = &props.player_state;
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
                {for chars.iter().enumerate().map(|(i, c)| html!{
                    <Character is_active={(i as u8) == active} char_state={c.clone()} hash={props.hash} />
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
    html! {
        <div class={classes!("char-part", if props.is_active { Some("is-active") } else { None })}>
            <h5>{char_card.name}</h5>
            <ul>
                <li class="char-elements">
                    {for char_state.applied.iter().map(|element| html!{
                        <Elem {element} />
                    })}
                </li>
                <li>{format!("HP: {}/{}", char_state.get_hp(), char_card.max_health)}</li>
                <li>{format!("Energy: {}/{}", char_state.get_energy(), char_card.max_energy)}</li>
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
}

impl PartialEq for StatusInfoProps {
    fn eq(&self, other: &Self) -> bool {
        self.status.name == other.status.name && self.state == other.state
    }
}

#[function_component(StatusInfo)]
fn status_info(StatusInfoProps { status, state }: &StatusInfoProps) -> Html {
    html! {
        <ul>
            {
                if status.usages.is_some() {
                    html!{ <li>{format!("Usages: {}", state.get_usages())}</li> }
                } else if status.duration_rounds.is_some() {
                    html!{ <li>{format!("Duration: {}", state.get_usages())}</li> }
                } else {
                    html!{}
                }
            }
        </ul>
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
        <span class={"card"}>
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
        <span class={format!("elem-{e:?}")}>
            {format!("{e:?}")}
        </span>
    }
}