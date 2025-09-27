use std::sync::atomic::{AtomicUsize, Ordering};

use petgraph::graph::NodeIndex;

use crate::{
    simulation::{
        executor::Executor,
        logging::{LogEntry, SimulationLog},
        state::State,
    },
    statistics::{
        roller::Roller,
        state_tree::{StateTree, StateTreeStats},
    },
    utils::ProtectedCell,
};

pub type Timestamp = chrono::DateTime<chrono::Utc>;

pub struct Integrator {
    pub max_combats: usize,
    pub combats_run: AtomicUsize,
    pub start_time: Timestamp,
    pub roller: Roller,
    pub state_tree: StateTree,
}

impl Integrator {
    pub fn new(max_combats: usize, roller: Roller, initial_state: State) -> Self {
        Self {
            max_combats,
            combats_run: AtomicUsize::new(0),
            start_time: chrono::Utc::now(),
            roller,
            state_tree: StateTree::new(initial_state),
        }
    }

    pub fn state_tree(&self) -> &StateTree {
        &self.state_tree
    }

    pub fn combats_run(&self) -> usize {
        self.combats_run.load(Ordering::Relaxed)
    }

    pub fn record_combat(&self) {
        self.combats_run.fetch_add(1, Ordering::Relaxed);
    }

    pub fn should_continue(&self) -> bool {
        self.combats_run() < self.max_combats
    }

    pub fn elapsed_time(&self) -> chrono::Duration {
        chrono::Utc::now() - self.start_time
    }

    pub fn combats_per_second(&self) -> f64 {
        let elapsed = self.elapsed_time().num_milliseconds() as f64 / 1000.0;
        if elapsed > 0.0 {
            self.combats_run() as f64 / elapsed
        } else {
            0.0
        }
    }

    pub fn run_one(&mut self) -> anyhow::Result<()> {
        let initial_state = self.state_tree.graph[self.state_tree.root].state.clone();
        let executor = Executor::new(self.roller.clone(), *initial_state);
        let current_node = self.state_tree.root;

        let mut cx = RunContext {
            state_tree: &mut self.state_tree,
            executor,
            current_node,
        };

        cx.run_combat()?;

        self.roller = cx.executor.roller;

        self.record_combat();

        Ok(())
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.start_time = chrono::Utc::now();
        while self.should_continue() {
            self.run_one()?;
        }
        Ok(())
    }

    pub fn compute_statistics(&self) -> StateTreeStats {
        self.state_tree.compute_statistics()
    }
}

struct RunContext<'a> {
    pub state_tree: &'a mut StateTree,
    pub executor: Executor,
    pub current_node: NodeIndex,
}

impl<'a> RunContext<'a> {
    pub fn apply_logs(&mut self, logs: SimulationLog) -> anyhow::Result<()> {
        for entry in logs {
            if let LogEntry::Transition(transition) = entry {
                let new_state = self.executor.state.get().clone();
                let new_node = self.state_tree.add_node(new_state);
                transition.apply(ProtectedCell::get_mut(&mut self.executor.state))?;
                self.state_tree
                    .add_edge(self.current_node, new_node, transition);
                self.current_node = new_node;
            }
        }

        Ok(())
    }

    pub fn run_combat(&mut self) -> anyhow::Result<()> {
        self.executor.begin_combat()?;
        let logs = self.executor.take_log();
        self.apply_logs(logs)?;

        while !self.executor.state.is_combat_over() {
            let still_going = self.executor.advance_turn()?;
            let logs = self.executor.take_log();
            self.apply_logs(logs)?;
            if !still_going {
                break;
            }
        }

        self.executor.end_combat()?;
        let logs = self.executor.take_log();
        self.apply_logs(logs)?;
        Ok(())
    }
}
