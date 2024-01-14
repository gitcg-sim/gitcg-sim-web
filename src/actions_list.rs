use gitcg_sim::{ids::*, prelude::*, types::card_defs::Cost};
use yew::prelude::*;

use crate::app::{AppAction, AppState, GameStateProp};

#[derive(Properties, PartialEq)]
pub struct ActionsListProps {
    pub app: UseReducerHandle<AppState>,
}

#[function_component(ActionsList)]
pub fn actions_list(props: &ActionsListProps) -> Html {
    let app = &props.app;
    let perform_action = |action: Input| {
        let app = app.clone();
        Callback::from(move |_: MouseEvent| {
            app.dispatch(AppAction::PerformAction(action));
        })
    };
    if let Some(winner) = app.game_state.winner() {
        return html! {
            <div class="winner-decided"><p>{"Winner decided: "}{format!("{}", winner)}</p></div>
        };
    }
    let acts = app.game_state.actions();
    html! {
        <div class="actions-list">
            <table>
                <thead>
                    <th>{"#"}</th>
                    <th>{"Name"}</th>
                    <th>{"Target"}</th>
                    <th style="min-width: 62px;">{"Cost"}</th>
                    <th>{"Action"}</th>
                </thead>
                <tbody>
                    {for acts.iter().enumerate().map({
                        let game_state = GameStateProp::new(&app.game_state.game_state);
                        let disabled = app.game_state.to_move() == Some(PlayerId::PlayerSecond);
                        move |(i, &action)| {
                            let onclick = perform_action(action);
                            html! {
                                <tr>
                                    <td>{format!("{}", i + 1)}</td>
                                    <td>
                                        <button {onclick} {disabled}>
                                            <ActionName {action} game_state={game_state.clone()} />
                                        </button>
                                    </td>
                                    <td><ActionTarget {action} game_state={game_state.clone()} /></td>
                                    <td><CostInfo cost={game_state.action_info(action).0} /></td>
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
                html! {
                    <span class="action action-cast-skill" title="Cast Skill">
                        {skill_id.get_skill().name}
                    </span>
                }
            }
            PlayerAction::PlayCard(card_id, _) => {
                html! {
                    <span class="action action-play-card" title="Play Card">
                        <span class="card-name">{card_id.get_card().name}</span>
                    </span>
                }
            }
            PlayerAction::ElementalTuning(card_id) => {
                html! {
                    <span class="action action-elemental-tuning" title="Elemental Tuning">
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
                html! { <span class="target-char">{get_char_name(player_id, i)}</span> }
            }
            PlayerAction::PlayCard(_, Some(CardSelection::OwnSummon(summon_id))) => {
                html! { <span class="own-summon">{summon_id.get_status().name}</span> }
            }
            PlayerAction::PlayCard(_, Some(CardSelection::OpponentSummon(summon_id))) => {
                html! { <span class="opp-summon">{summon_id.get_status().name}</span> }
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

#[derive(Properties, PartialEq)]
pub struct CostInfoProps {
    pub cost: Cost,
}

#[function_component(CostInfo)]
pub fn cost_info(props: &CostInfoProps) -> Html {
    let cost = props.cost;
    if cost.total_dice() == 0 {
        return html! {
            <span class="cost cost-zero">
                <span class="cost cost-aligned" title="Zero cost">{"0"}</span>
            </span>
        };
    }

    html! {
        <span class="cost">
            {cost.elem_cost.map(|(e, c)|
                html! {
                    <span
                        class={format!("cost cost-elem elem-{}", e.get_name())}
                        title={format!("Elemental cost: {}", e.get_name())}
                    >{c}</span>
                }
            )}
            {if cost.unaligned_cost > 0 {
                Some(html! {
                    <span class="cost cost-unaligned" title="Unaligned cost">{cost.unaligned_cost}</span>
                })
            } else { None }}
            {if cost.aligned_cost > 0 {
                Some(html! {
                    <span class="cost cost-aligned" title="Aligned cost">{cost.aligned_cost}</span>
                })
            } else { None }}
            {if cost.energy_cost > 0 {
                Some(html! {
                    <span class="cost cost-energy" title="Energy cost">{cost.energy_cost}</span>
                })
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
