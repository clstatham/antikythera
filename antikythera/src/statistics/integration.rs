use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use petgraph::graph::NodeIndex;

use crate::{
    simulation::{
        executor::Executor,
        logging::{LogEntry, SimulationLog},
        state::State,
    },
    statistics::{roller::Roller, state_tree::StateTree},
    utils::ProtectedCell,
};

pub type Timestamp = chrono::DateTime<chrono::Utc>;

pub struct Integrator {
    pub min_combats: usize,
    pub combats_run: Arc<AtomicUsize>,
    pub start_time: Timestamp,
    pub roller: Roller,
    pub initial_state: State,
}

impl Integrator {
    pub fn new(min_combats: usize, roller: Roller, initial_state: State) -> Self {
        Self {
            min_combats,
            combats_run: Arc::new(AtomicUsize::new(0)),
            start_time: chrono::Utc::now(),
            roller,
            initial_state,
        }
    }

    pub fn combats_run(&self) -> usize {
        self.combats_run.load(Ordering::Relaxed)
    }

    pub fn record_combat(&self) {
        self.combats_run.fetch_add(1, Ordering::Relaxed);
    }

    pub fn should_continue(&self) -> bool {
        self.combats_run() < self.min_combats
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

    pub fn run(&mut self) -> anyhow::Result<StateTree> {
        self.start_time = chrono::Utc::now();
        let mut state_tree = StateTree::new(self.initial_state.clone());
        let mut roller = self.roller.fork();
        while self.should_continue() {
            self.run_combat(roller.fork(), &mut state_tree)?;
        }

        Ok(state_tree)
    }

    pub fn run_combat(&mut self, roller: Roller, state_tree: &mut StateTree) -> anyhow::Result<()> {
        let mut executor = Executor::new(roller, self.initial_state.clone());
        executor.begin_combat()?;
        let logs = executor.take_log();
        let mut current_node = state_tree.root;
        self.apply_logs(&mut current_node, state_tree, &mut executor, logs)?;
        while !executor.state.is_combat_over() {
            self.advance_turn(&mut executor, state_tree, &mut current_node)?;
        }
        executor.end_combat()?;
        let logs = executor.take_log();
        self.apply_logs(&mut current_node, state_tree, &mut executor, logs)?;

        self.combats_run.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn advance_turn(
        &mut self,
        executor: &mut Executor,
        state_tree: &mut StateTree,
        current_node: &mut NodeIndex,
    ) -> anyhow::Result<bool> {
        let still_going = executor.advance_turn()?;
        let logs = executor.take_log();
        self.apply_logs(current_node, state_tree, executor, logs)?;
        Ok(still_going)
    }

    fn apply_logs(
        &mut self,
        current_node: &mut NodeIndex,
        state_tree: &mut StateTree,
        executor: &mut Executor,
        logs: SimulationLog,
    ) -> anyhow::Result<()> {
        for entry in logs {
            if let LogEntry::Transition(transition) = entry {
                let new_node = state_tree.add_node(&executor.state);
                transition.apply(ProtectedCell::get_mut(&mut executor.state))?;
                state_tree.add_edge(*current_node, new_node, transition);
                *current_node = new_node;
            }
        }

        Ok(())
    }
}
