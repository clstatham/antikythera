use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use petgraph::graph::NodeIndex;
use serde::{Deserialize, Serialize};

use crate::{
    prelude::{ActionEconomyUsage, Policy, RollSettings, Transition},
    simulation::{
        executor::Executor,
        logging::{LogEntry, SimulationLog},
        state::State,
    },
    statistics::{hook::Hook, roller::Roller, state_tree::StateTree},
    utils::ProtectedCell,
};

pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResults {
    pub state_tree: StateTree,
    pub combats_run: usize,
    pub elapsed: chrono::Duration,
    pub hook_metrics: Vec<(String, f64)>,
}

pub struct Integrator {
    pub min_combats: usize,
    pub combats_run: Arc<AtomicUsize>,
    pub start_time: Timestamp,
    pub roller: Roller,
    pub initial_state: State,
    pub hooks: Vec<Box<dyn Hook>>,
}

impl Integrator {
    pub fn new(min_combats: usize, roller: Roller, initial_state: State) -> Self {
        Self {
            min_combats,
            combats_run: Arc::new(AtomicUsize::new(0)),
            start_time: chrono::Utc::now(),
            roller,
            initial_state,
            hooks: Vec::new(),
        }
    }

    pub fn add_hook<H: Hook + 'static>(&mut self, hook: H) {
        self.hooks.push(Box::new(hook));
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

    pub fn run(&mut self) -> anyhow::Result<IntegrationResults> {
        for hook in &mut self.hooks {
            hook.on_integration_start(&self.initial_state);
        }
        let mut state_tree = StateTree::new(self.initial_state.clone());
        let mut roller = self.roller.fork();
        self.start_time = chrono::Utc::now();
        while self.should_continue() {
            self.run_combat(roller.fork(), &mut state_tree)?;
        }
        let elapsed = self.elapsed_time();

        for hook in &mut self.hooks {
            hook.on_integration_end();
        }
        let hook_metrics = self
            .hooks
            .iter()
            .flat_map(|hook| hook.metrics().into_iter())
            .collect();
        let results = IntegrationResults {
            state_tree,
            combats_run: self.combats_run(),
            elapsed,
            hook_metrics,
        };
        Ok(results)
    }

    pub fn run_combat(&mut self, roller: Roller, state_tree: &mut StateTree) -> anyhow::Result<()> {
        let mut executor = Executor::new(roller, self.initial_state.clone());
        // ROLL INITIATIVE!!!
        let mut initiative_rolls = BTreeMap::new();
        for actor in executor.state.actors.values() {
            let roll = actor.plan_initiative_roll(RollSettings::default());
            let result = roll.roll(&mut executor.roller)?;
            initiative_rolls.insert(actor.id, result.total);
        }

        executor.log(LogEntry::Transition(Transition::BeginCombat))?;

        for (actor_id, roll) in &initiative_rolls {
            executor.log(LogEntry::Transition(Transition::InitiativeRoll {
                actor: *actor_id,
                roll: *roll,
            }))?;
        }
        let logs = executor.take_log();
        let mut current_node = state_tree.root;
        self.apply_logs(&mut current_node, state_tree, &mut executor, logs)?;

        while !executor.state.is_combat_over() {
            self.advance_turn(&mut executor, state_tree, &mut current_node)?;
        }

        executor.log(LogEntry::Transition(Transition::EndCombat))?;

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
        if executor.state.initiative_order.is_empty() {
            return Ok(false);
        }

        if executor.state.is_combat_over() {
            return Ok(false);
        }

        // advance to next actor in initiative order
        executor.log(LogEntry::Transition(Transition::AdvanceInitiative))?;

        let logs = executor.take_log();
        self.apply_logs(current_node, state_tree, executor, logs)?;

        let current_actor_id =
            executor.state.initiative_order[executor.state.current_turn_index.unwrap()];

        let Some(current_actor) = executor.state.get_actor(current_actor_id) else {
            anyhow::bail!("Current actor not found in simulation state");
        };

        // dead actors skip their turn
        if current_actor.is_unconscious() || current_actor.is_dead() {
            return Ok(true);
        }

        executor.log(LogEntry::Transition(Transition::BeginTurn {
            actor: current_actor_id,
        }))?;

        for action_type in [ActionEconomyUsage::Action, ActionEconomyUsage::BonusAction] {
            let action_taken = executor.policy.take_action(
                action_type,
                current_actor_id,
                &executor.state,
                &mut executor.roller,
            )?;
            let action_logs = executor.evaluator.evaluate_action(
                current_actor_id,
                &action_taken,
                &executor.state,
                &mut executor.roller,
            )?;

            executor.extend_log(action_logs)?;

            for hook in &mut self.hooks {
                hook.on_action_executed(&executor.state, &action_taken);
            }
        }

        executor.log(LogEntry::Transition(Transition::EndTurn {
            actor: current_actor_id,
        }))?;

        let logs = executor.take_log();
        self.apply_logs(current_node, state_tree, executor, logs)?;
        Ok(true)
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
                state_tree.add_edge(*current_node, new_node, transition.clone());
                *current_node = new_node;

                match transition {
                    Transition::BeginCombat => {
                        for hook in &mut self.hooks {
                            hook.on_combat_start(&executor.state);
                        }
                    }
                    Transition::BeginTurn { actor } => {
                        for hook in &mut self.hooks {
                            hook.on_turn_start(&executor.state, actor, executor.state.turn);
                        }
                    }
                    Transition::EndTurn { actor } => {
                        for hook in &mut self.hooks {
                            hook.on_turn_end(&executor.state, actor, executor.state.turn);
                        }
                    }
                    Transition::EndCombat => {
                        for hook in &mut self.hooks {
                            hook.on_combat_end(&executor.state);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
