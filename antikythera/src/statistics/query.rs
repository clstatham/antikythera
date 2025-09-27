use crate::{simulation::state::State, statistics::state_tree::StateTree};

pub trait Query {
    type Output;
    fn query(&self, state_tree: &StateTree) -> anyhow::Result<Self::Output>;
}

impl<F, O> Query for F
where
    F: Fn(&StateTree) -> anyhow::Result<O> + 'static,
{
    type Output = O;

    fn query(&self, state_tree: &StateTree) -> anyhow::Result<Self::Output> {
        (self)(state_tree)
    }
}

/// A query that computes the probability of an ending state satisfying a given condition.
pub struct OutcomeConditionProbability {
    pub condition: Box<dyn Fn(&State) -> bool>,
}

impl OutcomeConditionProbability {
    pub fn new<F>(condition: F) -> Self
    where
        F: Fn(&State) -> bool + 'static,
    {
        Self {
            condition: Box::new(condition),
        }
    }
}

impl Query for OutcomeConditionProbability {
    type Output = f64;

    fn query(&self, state_tree: &StateTree) -> anyhow::Result<Self::Output> {
        let mut condition_hits = 0u64;
        let mut total_outgoing_hits = 0u64;

        state_tree.visit_states(true, |state, hits| {
            if (self.condition)(state) {
                condition_hits += hits;
            }
            total_outgoing_hits += hits;
            true
        });

        if total_outgoing_hits > 0 {
            Ok(condition_hits as f64 / total_outgoing_hits as f64)
        } else {
            Ok(0.0)
        }
    }
}
