
use gitcg_sim::{
    game_tree_search::*,
    ids::*,
    types::{game_state::*, input::*},
};
use yew::prelude::*;

use crate::{
    app::{AppAction, AppState, GameStateProp}
};

#[derive(Properties, PartialEq)]
pub struct ActionsListProps {
    pub app_state: UseReducerHandle<AppState>,
}

#[function_component(ActionsList)]
pub fn actions_list(props: &ActionsListProps) -> Html {
    let app_state = &props.app_state;
    let perform_action = |action: Input| {
        let app_state = app_state.clone();
        Callback::from(move |_: MouseEvent| {
            app_state.dispatch(AppAction::PerformAction(action));
        })
    };
    let acts = app_state.game_state.actions();
    html! {
        <div class="actions-list">
            <table>
                <thead>
                    <th>{"#"}</th>
                    <th>{"Name"}</th>
                    <th>{"Target"}</th>
                    <th>{"Cost"}</th>
                    <th>{"Action"}</th>
                </thead>
                <tbody>
                    {for acts.iter().enumerate().map({
                        let game_state = GameStateProp::new(&app_state.game_state.game_state);
                        move |(i, &action)| {
                            let onclick = perform_action(action);
                            html! {
                                <tr>
                                    <td>{format!("{}", i + 1)}</td>
                                    <td>
                                        <button {onclick}>
                                            <ActionName {action} game_state={game_state.clone()} />
                                        </button>
                                    </td>
                                    <td><ActionTarget {action} game_state={game_state.clone()} /></td>
                                    <td><ActionCost {action} game_state={game_state.clone()} /></td>
                                    <td><ActionType {action} game_state={game_state.clone()} /></td>
                                </tr>
                            }
                        }
                    })}
                </tbody>
            </table>
        </div>
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct ActionProps {
    pub action: Input,
    pub game_state: GameStateProp,
}

#[function_component(ActionName)]
pub fn action_name(props: &ActionProps) -> Html {
    match props.action {
        Input::FromPlayer(_, act) => match act {
            PlayerAction::CastSkill(skill_id) => {
                html! { <span class="action action-cast-skill">{skill_id.get_skill().name}</span> }
            }
            PlayerAction::PlayCard(card_id, _) => {
                html! {
                    <span class="action action-play-card">
                        <span class="card-name">{card_id.get_card().name}</span>
                    </span>
                }
            }
            PlayerAction::ElementalTuning(card_id) => {
                html! {
                    <span class="action action-elemental-tuning">
                        {"ET: "}
                        <span class="card-name">{card_id.get_card().name}</span>
                    </span>
                }
            }
            PlayerAction::SwitchCharacter(_) | PlayerAction::PostDeathSwitch(_) => {
                html! { <span class="action action-switch">{"Switch"}</span> }
            }
            PlayerAction::EndRound => {
                html! { <span class="action action-end-round">{"End Round"}</span> }
            }
        },
        Input::NoAction | Input::NondetResult(..) => html! { "" },
    }
}

#[function_component(ActionTarget)]
pub fn action_target(props: &ActionProps) -> Html {
    let get_char_name = |player_id, i| {
        props
            .game_state
            .get_player(player_id)
            .get_character_card(i)
            .name
    };

    match props.action {
        Input::FromPlayer(player_id, act) => match act {
            PlayerAction::PlayCard(_, Some(CardSelection::OwnCharacter(i))) => {
                html! { <span class="taret-char">{get_char_name(player_id, i)}</span> }
            }
            PlayerAction::SwitchCharacter(i) | PlayerAction::PostDeathSwitch(i) => {
                html! { <span class="target-char">{get_char_name(player_id, i)}</span> }
            }
            PlayerAction::PlayCard(_, None)
            | PlayerAction::CastSkill(..)
            | PlayerAction::ElementalTuning(..)
            | PlayerAction::EndRound => {
                html! { <span /> }
            }
        },
        Input::NoAction | Input::NondetResult(..) => html! { "" },
    }
}


#[function_component(ActionCost)]
pub fn action_cost(props: &ActionProps) -> Html {
    let game_state = &props.game_state.0;
    let cost = game_state.action_info(props.action).0;
    html! {
        <span class="cost">
            {cost.elem_cost.map(|(e, c)|
                html! { <span class={format!("cost cost-elem elem-{e:?}")}>{c}</span> }
            )}
            {if cost.unaligned_cost > 0 {
                Some(html! { <span class="cost cost-unaligned">{cost.unaligned_cost}</span> })
            } else { None }}
            {if cost.aligned_cost > 0 {
                Some(html! { <span class="cost cost-aligned">{cost.aligned_cost}</span> })
            } else { None }}
            {if cost.energy_cost > 0 {
                Some(html! { <span class="cost cost-energy">{cost.energy_cost}</span> })
            } else { None }}
        </span>
    }
}

#[function_component(ActionType)]
pub fn action_type(props: &ActionProps) -> Html {
    let game_state = &props.game_state.0;
    let is_fast_action = game_state.action_info(props.action).1;
    if is_fast_action {
        html! {
            <span class="action-type action-type-fast-action">{"Fast"}</span>
        }
    } else {
        html! {
            <span class="action-type action-type-combat-action">{"Combat"}</span>
        }
    }
}