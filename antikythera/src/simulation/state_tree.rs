use std::{
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    num::NonZeroU64,
};

use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};

use crate::simulation::{state::State, transition::Transition};

pub type NodeIndex = u32;
pub type EdgeIndex = u32;

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
#[repr(transparent)]
#[serde(transparent)]
pub struct Node {
    pub hits: NonZeroU64,
}

impl Node {
    pub fn new() -> Self {
        Self {
            hits: NonZeroU64::MIN, // Start with 1 hit
        }
    }
}

impl Default for Node {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub transition: Transition,
    pub hits: NonZeroU64,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(transparent)]
pub struct EdgeKey(u64);

impl EdgeKey {
    #[inline]
    pub fn new(source: NodeIndex, to: NodeIndex) -> Self {
        EdgeKey(((source as u64) << 32) | (to as u64))
    }

    #[inline]
    pub fn source(&self) -> NodeIndex {
        (self.0 >> 32) as NodeIndex
    }

    #[inline]
    pub fn target(&self) -> NodeIndex {
        (self.0 & 0xFFFFFFFF) as NodeIndex
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StateTree {
    initial_state: State,
    root: NodeIndex,
    nodes: Vec<NonZeroU64>,
    total_node_hits: u64,
    total_edge_hits: u64,
    state_cache: HashMap<StateHash, NodeIndex, NoHashBuildHasher>,
    edge_cache: BTreeMap<EdgeKey, Edge>,
    neighbors: Vec<Vec<NodeIndex>>,
}

impl StateTree {
    pub fn new(initial_state: State) -> Self {
        let mut this = Self {
            initial_state,
            root: 0,
            nodes: Vec::new(),
            total_node_hits: 0,
            total_edge_hits: 0,
            state_cache: HashMap::default(),
            edge_cache: BTreeMap::default(),
            neighbors: Vec::new(),
        };
        this.root = this.add_node(StateHash::hash_state(&this.initial_state));
        this
    }

    pub fn add_node(&mut self, state_hash: StateHash) -> NodeIndex {
        self.total_node_hits = self.total_node_hits.saturating_add(1);

        // Check if the node already exists
        if let Some(&existing_index) = self.state_cache.get(&state_hash)
            && let Some(node_hits) = self.nodes.get_mut(existing_index as usize)
        {
            // Increment hits if it exists
            *node_hits = node_hits.saturating_add(1);

            existing_index
        } else {
            // Add the new node
            let node = self.nodes.len() as NodeIndex;
            self.nodes.push(NonZeroU64::MIN); // Start with 1 hit
            self.state_cache.insert(state_hash, node);

            node
        }
    }

    pub fn add_edge(
        &mut self,
        from: NodeIndex,
        to: NodeIndex,
        transition: Transition,
    ) -> Option<EdgeKey> {
        // Check if the edge already exists
        let key = EdgeKey::new(from, to);
        if let Some(existing_edge) = self.edge_cache.get_mut(&key) {
            debug_assert_eq!(
                existing_edge.transition, transition,
                "Discontinuity in transition graph detected: existing transition does not match new transition for edge from {:?} to {:?}",
                from, to
            );
            // Increment hits if it exists
            existing_edge.hits = existing_edge.hits.saturating_add(1);
            self.total_edge_hits = self.total_edge_hits.saturating_add(1);
            Some(key)
        } else {
            // Add the new edge
            let edge = Edge {
                transition,
                hits: NonZeroU64::MIN, // Start with 1 hit
            };
            self.edge_cache.insert(key, edge);
            self.total_edge_hits = self.total_edge_hits.saturating_add(1);

            // Update neighbors
            if let Some(neighbors) = self.neighbors.get_mut(from as usize) {
                neighbors.push(to);
            } else {
                // Ensure the neighbors vector is large enough
                self.neighbors.resize((from + 1) as usize, Vec::new());
                self.neighbors[from as usize].push(to);
            }

            Some(key)
        }
    }

    pub fn root(&self) -> NodeIndex {
        self.root
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edge_cache.len()
    }

    pub fn neighbors<'a>(&'a self, node: NodeIndex) -> impl Iterator<Item = NodeIndex> + 'a {
        self.neighbors
            .get(node as usize)
            .into_iter()
            .flat_map(|v| v.iter().copied())
    }

    pub fn get_node_hits(&self, index: NodeIndex) -> Option<NonZeroU64> {
        self.nodes.get(index as usize).copied()
    }

    pub fn get_edge(&self, from: NodeIndex, to: NodeIndex) -> Option<&Edge> {
        let key = EdgeKey::new(from, to);
        self.edge_cache.get(&key)
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
            self.neighbors(node).next().is_none()
        } else {
            true
        };

        // Visit the state at the current node
        let keep_going = if should_visit {
            let hits = self.get_node_hits(node).map_or(0, |h| h.get());
            visitor(state, hits)
        } else {
            true
        };
        if !keep_going {
            return;
        }

        for neighbor in self.neighbors(node) {
            // Apply the transition to get the new state
            if let Some(edge) = self.get_edge(node, neighbor) {
                let mut new_state = state.clone();
                if let Err(e) = edge.transition.apply(&mut new_state) {
                    log::error!("Error applying transition: {:?}", e);
                    continue;
                }
                self.visit_states_recursive(externals_only, neighbor, &new_state, visited, visitor);
            }
        }
    }
}
