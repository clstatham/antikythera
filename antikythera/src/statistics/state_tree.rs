use std::{collections::VecDeque, fmt::Debug, num::NonZeroU64};

use petgraph::prelude::*;
use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};

use crate::simulation::{state::State, transition::Transition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub state: Box<State>, // Boxed to reduce size
    pub hits: NonZeroU64,
}

impl Node {
    pub fn new(state: State) -> Self {
        Self {
            state: Box::new(state),
            hits: NonZeroU64::MIN, // Start with 1 hit
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.state == other.state
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub transition: Transition,
    pub hits: NonZeroU64,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct StateTree {
    pub graph: DiGraph<Node, Edge>,
    pub root: NodeIndex,
    pub total_node_hits: u64,
    pub total_edge_hits: u64,
    #[serde(skip)]
    state_cache: FxHashMap<State, NodeIndex>,
}

impl StateTree {
    pub fn new(initial_state: State) -> Self {
        let initial_node = Node::new(initial_state.clone());
        let mut graph = DiGraph::new();
        let root = graph.add_node(initial_node);
        let mut state_cache = FxHashMap::default();
        state_cache.insert(initial_state, root);
        Self {
            graph,
            root,
            total_node_hits: 0,
            total_edge_hits: 0,
            state_cache,
        }
    }

    pub fn add_node(&mut self, state: State) -> NodeIndex {
        let node = Node::new(state);
        // Check if the node already exists
        if let Some(&existing_index) = self.state_cache.get(&node.state) {
            // Increment hits if it exists
            if let Some(existing_node) = self.graph.node_weight_mut(existing_index) {
                existing_node.hits = existing_node.hits.saturating_add(1);
                self.total_node_hits = self.total_node_hits.saturating_add(1);
            }
            existing_index
        } else {
            // Add the new node
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
        if let Some(existing_edge) = self.graph.find_edge(from, to) {
            // Increment hits if it exists
            if let Some(edge) = self.graph.edge_weight_mut(existing_edge) {
                edge.hits = edge.hits.saturating_add(1);
                self.total_edge_hits = self.total_edge_hits.saturating_add(1);
            }
            Some(existing_edge)
        } else {
            // Add the new edge
            Some(self.graph.add_edge(
                from,
                to,
                Edge {
                    transition,
                    hits: NonZeroU64::MIN, // Start with 1 hit
                },
            ))
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

    pub fn iter_nodes(&self) -> impl Iterator<Item = (NodeIndex, &Node)> {
        self.graph.node_indices().map(move |i| (i, &self.graph[i]))
    }

    pub fn compute_statistics(&self) -> StateTreeStats {
        let total_nodes = self.graph.node_count();
        let total_edges = self.graph.edge_count();
        let average_branching_factor = if total_nodes > 0 {
            total_edges as f64 / total_nodes as f64
        } else {
            0.0
        };

        // Compute max depth using BFS
        let mut max_depth = 0;
        let mut visited = FxHashSet::default();
        let mut queue = VecDeque::new();
        queue.push_back((self.root, 0));
        visited.insert(self.root);

        while let Some((node, depth)) = queue.pop_front() {
            max_depth = max_depth.max(depth);
            for neighbor in self.graph.neighbors(node) {
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, depth + 1));
                }
            }
        }

        // Build probability graph
        let mut probability_graph = DiGraph::new();
        let mut node_map = FxHashMap::default();

        for (index, node) in self.iter_nodes() {
            // probability of reaching this node from its parent
            let probability = if index == self.root {
                1.0
            } else {
                let mut incoming_hits = 0u64;
                for edge in self.graph.edges_directed(index, Incoming) {
                    incoming_hits += edge.weight().hits.get();
                }
                if incoming_hits > 0 {
                    node.hits.get() as f64 / incoming_hits as f64
                } else {
                    0.0
                }
            };
            let stat_node = StateTreeStatNode {
                id: index,
                hits: node.hits.get(),
                probability,
            };
            let stat_index = probability_graph.add_node(stat_node);
            node_map.insert(index, stat_index);
        }

        for edge in self.graph.edge_references() {
            let from = edge.source();
            let to = edge.target();
            let edge_weight = edge.weight();
            let from_stat_index = node_map[&from];
            let to_stat_index = node_map[&to];
            // probability of this transition given the from node
            let mut outgoing_hits = 0u64;
            for out_edge in self.graph.edges_directed(from, Outgoing) {
                outgoing_hits += out_edge.weight().hits.get();
            }
            let probability = if outgoing_hits > 0 {
                edge_weight.hits.get() as f64 / outgoing_hits as f64
            } else {
                0.0
            };
            // let probability = edge_weight.hits.get() as f64 / self.graph[from].hits.get() as f64;
            let stat_edge = StateTreeStatEdge {
                transition: edge_weight.transition.clone(),
                hits: edge_weight.hits.get(),
                probability,
            };
            probability_graph.add_edge(from_stat_index, to_stat_index, stat_edge);
        }

        StateTreeStats {
            total_nodes,
            total_edges,
            average_branching_factor,
            max_depth,
            probability_graph,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTreeStatNode {
    pub id: NodeIndex,
    pub hits: u64,
    pub probability: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct StateTreeStatEdge {
    pub transition: Transition,
    pub hits: u64,
    pub probability: f64,
}

impl Debug for StateTreeStatEdge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}%", self.probability * 100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTreeStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub average_branching_factor: f64,
    pub max_depth: usize,
    pub probability_graph: DiGraph<StateTreeStatNode, StateTreeStatEdge>,
}

impl StateTreeStats {
    pub fn print_summary(&self) {
        println!("State Tree Statistics:");
        println!("Total Nodes: {}", self.total_nodes);
        println!("Total Edges: {}", self.total_edges);
        println!(
            "Average Branching Factor: {:.2}",
            self.average_branching_factor
        );
        println!("Max Depth: {}", self.max_depth);
    }

    pub fn write_json(&self, path: &str) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn write_dot(&self, path: &str) -> anyhow::Result<()> {
        let dot = petgraph::dot::Dot::with_config(
            &self.probability_graph,
            &[petgraph::dot::Config::NodeIndexLabel],
        );
        let mut file = std::fs::File::create(path)?;
        use std::io::Write;
        write!(file, "{:?}", dot)?;
        Ok(())
    }
}
