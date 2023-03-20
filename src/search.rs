use instant::Instant;
use std::rc::Rc;
use yew_agent::*;

use gitcg_sim::{
    game_tree_search::*,
    mcts::{MCTSConfig, MCTS},
    types::game_state::*,
};

use serde::{Deserialize, Serialize};

use crate::app::G;

#[derive(Serialize, Deserialize)]
pub struct WorkerMessage {
    maximize_player: PlayerId,
    game_state: G,
    time_ms_per_step: u32,
    steps: u32,
}

pub struct SearchWorker {
    pub link: WorkerLink<Self>,
    pub search: MCTS<G>,
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

const TIME_LIMIT_MS: u32 = 500;

impl Worker for SearchWorker {
    type Reach = Public<Self>;

    type Message = ();

    type Input = SearchAction;

    type Output = SearchReturn;

    fn create(link: WorkerLink<Self>) -> Self {
        let config = MCTSConfig::new(TIME_LIMIT_MS, 8.0, 8192, false, 40, 100, false);
        Self {
            link,
            search: MCTS::new(config),
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
                    break 'a
                };
                if search_steps.steps_remaining == 0 {
                    gloo::console::log!(format!(
                        "Finish, PV = {:?}",
                        self.solution.as_ref().map(|s| s
                            .pv
                            .clone()
                            .into_iter()
                            .collect::<Vec<_>>())
                    ));
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
                res.update(&res1);
                self.search_steps = Some(search_steps);
                self.solution = Some(res);
                self.link
                    .respond(id, SearchReturn(false, self.solution.clone(), t));
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
