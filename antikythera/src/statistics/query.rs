use petgraph::prelude::*;

use crate::{
    simulation::state::State,
    statistics::state_tree::{StateTree, StateTreeStats},
};

pub trait Query {
    type Output;
    fn query(
        &self,
        state_tree: &StateTree,
        statistics: &StateTreeStats,
    ) -> anyhow::Result<Self::Output>;
}

impl<F, O> Query for F
where
    F: Fn(&StateTree, &StateTreeStats) -> anyhow::Result<O> + 'static,
{
    type Output = O;

    fn query(
        &self,
        state_tree: &StateTree,
        statistics: &StateTreeStats,
    ) -> anyhow::Result<Self::Output> {
        (self)(state_tree, statistics)
    }
}

/// A query that computes the probability of reaching a given state ID.
pub fn state_probability(
    _state_tree: &StateTree,
    stats: &StateTreeStats,
    state_id: NodeIndex,
) -> anyhow::Result<f64> {
    if let Some(node) = stats.probability_graph.node_weight(state_id) {
        Ok(node.probability)
    } else {
        Ok(0.0)
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

    fn query(
        &self,
        state_tree: &StateTree,
        _stats: &StateTreeStats,
    ) -> anyhow::Result<Self::Output> {
        let mut condition_hits = 0u64;
        let mut total_outgoing_hits = 0u64;

        for node_index in state_tree.graph.externals(Outgoing) {
            let node = &state_tree.graph[node_index];
            total_outgoing_hits += node.hits.get();
            let state = state_tree.resolve_state(node_index).unwrap();
            if (self.condition)(&state) {
                condition_hits += node.hits.get();
            }
        }

        if total_outgoing_hits > 0 {
            Ok(condition_hits as f64 / total_outgoing_hits as f64)
        } else {
            Ok(0.0)
        }
    }
}
