use gloo::utils::format::JsValueSerdeExt;
use instant::Instant;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use yew_agent::*;

use gitcg_sim::{
    game_tree_search::*,
    mcts::{policy::DefaultEvalPolicy, MCTSConfig, MCTS},
    training::policy::PolicyNetwork,
    types::game_state::*,
};

use serde::{Deserialize, Serialize};

use crate::app::{describe_action_with_player, G};

#[derive(Serialize, Deserialize)]
pub struct WorkerMessage {
    maximize_player: PlayerId,
    game_state: G,
    time_ms_per_step: u32,
    steps: u32,
}

pub struct SearchWorker {
    pub link: WorkerLink<Self>,
    pub search: MCTS<G, DefaultEvalPolicy, PolicyNetwork>,
    pub search_steps: Option<SearchSteps>,
    pub solution: Option<SearchResult<G>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum SearchAction {
    Start {
        maximize_player: PlayerId,
        game_state: Rc<G>,
        steps: u32,
    },
    Step,
    Abandon,
    SetConfig(MCTSConfig),
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SearchReturn(pub bool, pub Option<SearchResult<G>>, pub u128);

#[derive(Clone)]
pub struct SearchSteps {
    pub total_time_ms: u128,
    pub steps_remaining: u32,
    pub maximize_player: PlayerId,
    pub game_state: G,
}

const TIME_LIMIT_MS: u128 = 500;

const DEFAULT_CONFIG: MCTSConfig = {
    let c = 2.0;
    let tt_size_mb = 32;
    let parallel = false;
    let random_playout_iters = 10;
    let random_playout_cutoff = 20;
    let random_playout_bias = Some(10.0);
    let debug = false;
    MCTSConfig {
        c,
        tt_size_mb,
        parallel,
        random_playout_iters,
        random_playout_cutoff,
        random_playout_bias,
        debug,
        limits: Some(SearchLimits {
            max_time_ms: Some(TIME_LIMIT_MS),
            max_positions: None,
        }),
    }
};

impl Worker for SearchWorker {
    type Reach = Public<Self>;

    type Message = ();

    type Input = SearchAction;

    type Output = SearchReturn;

    fn create(link: WorkerLink<Self>) -> Self {
        gloo::console::log!(
            "Worker initialized, config: ",
            JsValue::from_serde::<MCTSConfig>(&DEFAULT_CONFIG).unwrap()
        );
        Self {
            link,
            search: MCTS::new_with_eval_policy_and_selection_policy(
                DEFAULT_CONFIG,
                Default::default(),
                PolicyNetwork::new(),
            ),
            search_steps: None,
            solution: None,
        }
    }

    fn handle_input(&mut self, msg: Self::Input, id: HandlerId) {
        match msg {
            SearchAction::Start {
                maximize_player,
                game_state,
                steps,
            } => {
                self.search_steps = Some(SearchSteps {
                    steps_remaining: steps,
                    game_state: game_state.as_ref().clone(),
                    maximize_player,
                    total_time_ms: 0,
                });
                self.solution = None;
                self.link.respond(id, SearchReturn::default());
            }
            SearchAction::Abandon => {
                self.search_steps = None;
                self.solution = None;
                self.link.respond(id, SearchReturn::default());
            }
            SearchAction::Step => 'a: {
                let Some(mut search_steps) = self.search_steps.clone() else {
                    break 'a;
                };
                if search_steps.steps_remaining == 0 {
                    if let Some((_, root)) = self.search.root {
                        if let Some((root, initial_state)) = self
                            .search
                            .tree
                            .get(root)
                            .map(|root_node| (root_node.token(), root_node.data.state.clone()))
                        {
                            gloo::console::log!(format!(
                                "Search Finish: Principal Variation = {:?}",
                                self.solution.as_ref().map(|s| s
                                    .pv
                                    .clone()
                                    .map(|action| describe_action_with_player(
                                        &initial_state,
                                        action
                                    ))
                                    .collect::<Vec<_>>())
                            ));
                            gloo::console::log!(
                                "MCTS Tree: ",
                                JsValue::from_serde(&self.search.dump_tree(root, 4, &|action| {
                                    describe_action_with_player(&initial_state, action)
                                }))
                                .unwrap_or_default()
                            );
                            gloo::console::log!("-------------------");
                        }
                    }
                    self.link.respond(
                        id,
                        SearchReturn(true, self.solution.clone(), search_steps.total_time_ms),
                    );
                    break 'a;
                }

                let t0 = Instant::now();
                let mut res = self
                    .search
                    .search(&search_steps.game_state, search_steps.maximize_player);
                gloo::console::log!(format!("Step {:?}", res.pv.head()));
                gloo::console::log!(format!(
                    "Root: {}",
                    self.search
                        .root
                        .and_then(|(_, r)| self.search.tree.get(r))
                        .map(|d| format!("{:?}", d.data))
                        .unwrap_or_default()
                ));
                let dt = (Instant::now() - t0).as_nanos();

                let res1 = self.solution.clone().unwrap_or_default();

                search_steps.total_time_ms += dt;
                search_steps.steps_remaining -= 1;
                let t = search_steps.total_time_ms;
                {
                    res.counter.add_in_place(&res1.counter);
                    if res1.pv.len() >= res.pv.len() {
                        res.pv = res1.pv;
                        res.eval = res1.eval;
                    }
                }
                self.search_steps = Some(search_steps);
                self.solution = Some(res);
                self.link
                    .respond(id, SearchReturn(false, self.solution.clone(), t));
            }
            SearchAction::SetConfig(c) => {
                self.search.config = c;
            }
        }
    }

    fn update(&mut self, _msg: Self::Message) {}

    fn name_of_resource() -> &'static str {
        "worker.js"
    }

    fn resource_path_is_relative() -> bool {
        true
    }
}
