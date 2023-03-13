use instant::Instant;
use std::{ops::Deref, rc::Rc, cell::RefCell};

use gitcg_sim::{
    game_tree_search::{*, Game},
    types::{game_state::*}, mcts::{MCTS},
};

use yew::prelude::*;

#[derive(Clone)]
pub enum SearchAction<G: Game> {
    Start { maximize_player: PlayerId, game_state: Rc<G>, time_ms_per_step: u32, steps: u32 },
    Step,
    Abandon,
}

#[derive(Clone)]
pub struct SearchSteps<G: Game> {
    pub total_time_ms: u128,
    pub time_ms_per_step: u32,
    pub steps_remaining: u32,
    pub maximize_player: PlayerId,
    pub game_state: Rc<G>,
}

pub struct SearchState<G: Game> {
    pub search: Rc<RefCell<MCTS<G>>>,
    pub search_steps: Option<SearchSteps<G>>,
    pub solution: Option<SearchResult<G>>,
}

impl<G: Game> Clone for SearchState<G> {
    fn clone(&self) -> Self {
        Self { search: self.search.clone(), search_steps: self.search_steps.clone(), solution: self.solution.clone() }
    }
}


impl<G: Game> SearchState<G> {
    pub fn new(search: MCTS<G>) -> Self {
        let search = Rc::new(RefCell::new(search));
        Self { search, search_steps: None, solution: None }
    }
}

impl<G: Game> Reducible for SearchState<G> {
    type Action = SearchAction<G>;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        let mut next: Self = self.deref().clone();
        match action {
            Self::Action::Abandon => {
                next.search_steps = None
            },
            Self::Action::Start { time_ms_per_step, steps, game_state, maximize_player } => {
                next.search_steps = Some(SearchSteps {
                    time_ms_per_step, steps_remaining: steps, game_state, maximize_player, total_time_ms: 0,
                });
                next.solution = Default::default();
            },
            Self::Action::Step => 'a: {
                let Some(search_steps) = &self.search_steps else {
                    break 'a
                };
                if search_steps.steps_remaining == 0 {
                    break 'a
                }
                let mut search_steps = search_steps.clone();
                let mut search = next.search.deref().borrow_mut();
                let now = Instant::now();
                let mut res = search.search(&search_steps.game_state, search_steps.maximize_player);
                let dt = (Instant::now() - now).as_millis();

                let res1 = next.solution.unwrap_or_default();
                search_steps.total_time_ms += dt;
                search_steps.steps_remaining -= 1;
                res.update(&res1);
                next.search_steps = Some(search_steps);
                next.solution = Some(res);
            },
        }
        Rc::new(next)
    }

}
