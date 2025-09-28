use std::{collections::HashMap, fmt::Debug, num::NonZeroU64};

use petgraph::prelude::*;
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::simulation::{state::State, transition::Transition};

#[derive(Default)]
struct NoHashHasher(u64);

impl std::hash::Hasher for NoHashHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write_u64(&mut self, i: u64) {
        self.0 = i;
    }

    fn write(&mut self, _bytes: &[u8]) {
        #[cfg(debug_assertions)]
        panic!("NoHashHasher only supports write_u64");
    }
}

type NoHashBuildHasher = std::hash::BuildHasherDefault<NoHashHasher>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StateHash(u64);

impl StateHash {
    pub fn hash_state(state: &State) -> Self {
        use std::hash::{Hash, Hasher};
        let mut hasher = rustc_hash::FxHasher::default();
        state.hash(&mut hasher);
        StateHash(hasher.finish())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub state_hash: StateHash,
    pub hits: NonZeroU64,
}

impl Node {
    pub fn new(state_hash: StateHash) -> Self {
        Self {
            state_hash,
            hits: NonZeroU64::MIN, // Start with 1 hit
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.state_hash == other.state_hash
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub transition: Transition,
    pub hits: NonZeroU64,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[repr(transparent)]
struct EdgeKey(u64);

impl EdgeKey {
    #[inline]
    fn new(from: NodeIndex, to: NodeIndex) -> Self {
        // Combine the two NodeIndex values into a single u64
        EdgeKey(((from.index() as u64) << 32) | (to.index() as u64))
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StateTree {
    pub initial_state: State,
    pub graph: DiGraph<Node, Edge>,
    pub root: NodeIndex,
    pub total_node_hits: u64,
    pub total_edge_hits: u64,
    state_cache: HashMap<StateHash, NodeIndex, NoHashBuildHasher>,
    edge_cache: HashMap<EdgeKey, EdgeIndex, NoHashBuildHasher>,
}

impl StateTree {
    pub fn new(initial_state: State) -> Self {
        let initial_state_hash = StateHash::hash_state(&initial_state);
        let initial_node = Node::new(initial_state_hash);
        let mut graph = DiGraph::new();
        let root = graph.add_node(initial_node);
        let mut state_cache = HashMap::default();
        state_cache.insert(initial_state_hash, root);
        Self {
            initial_state,
            graph,
            root,
            total_node_hits: 0,
            total_edge_hits: 0,
            state_cache,
            edge_cache: HashMap::default(),
        }
    }

    pub fn add_node(&mut self, state: &State) -> NodeIndex {
        // Check if the node already exists
        let state_hash = StateHash::hash_state(state);
        if let Some(&existing_index) = self.state_cache.get(&state_hash) {
            // Increment hits if it exists
            if let Some(existing_node) = self.graph.node_weight_mut(existing_index) {
                existing_node.hits = existing_node.hits.saturating_add(1);
                self.total_node_hits = self.total_node_hits.saturating_add(1);
            }
            existing_index
        } else {
            // Add the new node
            let node = Node::new(state_hash);
            self.graph.add_node(node)
        }
    }

    pub fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        transition: Transition,
    ) -> Option<EdgeIndex> {
        // Check if the edge already exists
        let key = EdgeKey::new(from, to);
        if let Some(&existing_edge) = self.edge_cache.get(&key) {
            // Increment hits if it exists
            if let Some(edge) = self.graph.edge_weight_mut(existing_edge) {
                edge.hits = edge.hits.saturating_add(1);
                self.total_edge_hits = self.total_edge_hits.saturating_add(1);
            }
            Some(existing_edge)
        } else {
            // Add the new edge
            let edge = self.graph.add_edge(
                from,
                to,
                Edge {
                    transition,
                    hits: NonZeroU64::MIN, // Start with 1 hit
                },
            );
            self.edge_cache.insert(key, edge);
            Some(edge)
        }
    }

    pub fn get_node(&self, index: NodeIndex) -> Option<&Node> {
        self.graph.node_weight(index)
    }

    pub fn get_edge(&self, from: NodeIndex, to: NodeIndex) -> Option<&Edge> {
        self.graph
            .find_edge(from, to)
            .and_then(|e| self.graph.edge_weight(e))
    }

    pub fn increment_node_hits(&mut self, index: NodeIndex) {
        if let Some(node) = self.graph.node_weight_mut(index) {
            node.hits = node.hits.saturating_add(1);
            self.total_node_hits = self.total_node_hits.saturating_add(1);
        }
    }

    pub fn increment_edge_hits(&mut self, from: NodeIndex, to: NodeIndex) {
        if let Some(edge_index) = self.graph.find_edge(from, to)
            && let Some(edge) = self.graph.edge_weight_mut(edge_index)
        {
            edge.hits = edge.hits.saturating_add(1);
            self.total_edge_hits = self.total_edge_hits.saturating_add(1);
        }
    }

    pub fn resolve_state(&self, node: NodeIndex) -> Option<State> {
        let mut state = self.initial_state.clone();
        if let Some((_, path)) = petgraph::algo::astar(
            &self.graph,
            self.root,
            |finish| finish == node,
            |_| 1,
            |_| 0,
        ) {
            for node in path.windows(2) {
                if let [from, to] = node {
                    if let Some(edge) = self.graph.find_edge(*from, *to)
                        && let Some(edge_weight) = self.graph.edge_weight(edge)
                    {
                        if let Err(e) = edge_weight.transition.apply(&mut state) {
                            log::error!("Error applying transition: {:?}", e);
                            return None;
                        }
                    } else {
                        log::error!("Edge not found from {:?} to {:?}", from, to);
                        return None;
                    }
                } else {
                    log::error!("Invalid path segment: {:?}", node);
                    return None;
                }
            }
        }
        Some(state)
    }

    pub fn visit_states<F>(&self, externals_only: bool, mut visitor: F)
    where
        F: FnMut(&State, u64) -> bool,
    {
        self.visit_states_recursive(
            externals_only,
            self.root,
            &self.initial_state,
            &mut FxHashSet::default(),
            &mut visitor,
        )
    }

    fn visit_states_recursive<F>(
        &self,
        externals_only: bool,
        node: NodeIndex,
        state: &State,
        visited: &mut FxHashSet<NodeIndex>,
        visitor: &mut F,
    ) where
        F: FnMut(&State, u64) -> bool,
    {
        if !visited.insert(node) {
            return; // Already visited
        }

        let should_visit = if externals_only {
            self.graph.neighbors(node).next().is_none()
        } else {
            true
        };

        // Visit the state at the current node
        let keep_going = if should_visit {
            visitor(state, self.graph[node].hits.get())
        } else {
            true
        };
        if !keep_going {
            return;
        }

        for neighbor in self.graph.neighbors(node) {
            // Apply the transition to get the new state
            if let Some(edge) = self.graph.find_edge(node, neighbor)
                && let Some(edge_weight) = self.graph.edge_weight(edge)
            {
                let mut new_state = state.clone();
                if let Err(e) = edge_weight.transition.apply(&mut new_state) {
                    log::error!("Error applying transition: {:?}", e);
                    continue;
                }
                self.visit_states_recursive(externals_only, neighbor, &new_state, visited, visitor);
            }
        }
    }
}
