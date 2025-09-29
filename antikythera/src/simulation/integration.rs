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
    prelude::{
        Action, ActionEconomyUsage, ActionTaken, ActorId, ItemInner, RollSettings, Transition,
    },
    rules::actions::{AttackAction, UnarmedStrikeAction},
    simulation::{hook::Hook, roller::Roller, state::State, state_tree::StateTree},
    utils::ProtectedCell,
};

pub type Timestamp = chrono::DateTime<chrono::Utc>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResults {
    pub state_tree: StateTree,
    pub combats_run: usize,
    pub elapsed_time: chrono::Duration,
    pub hook_metrics: Vec<(String, f64)>,
}

impl IntegrationResults {
    pub fn combats_per_second(&self) -> f64 {
        let secs = self.elapsed_time.num_milliseconds() as f64 / 1000.0;
        if secs > 0.0 {
            self.combats_run as f64 / secs
        } else {
            0.0
        }
    }
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

    fn record_combat(&self) {
        self.combats_run.fetch_add(1, Ordering::Relaxed);
    }

    pub fn should_continue(&self) -> bool {
        self.combats_run() < self.min_combats
    }

    pub fn elapsed_time(&self) -> chrono::Duration {
        chrono::Utc::now() - self.start_time
    }

    pub fn run(&mut self) -> anyhow::Result<IntegrationResults> {
        for hook in &mut self.hooks {
            hook.on_integration_start(&self.initial_state);
        }
        let mut state_tree = StateTree::new(self.initial_state.clone());
        self.start_time = chrono::Utc::now();
        while self.should_continue() {
            self.run_combat(&mut state_tree)?;
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
            elapsed_time: elapsed,
            hook_metrics,
        };
        Ok(results)
    }

    pub fn run_combat(&mut self, state_tree: &mut StateTree) -> anyhow::Result<()> {
        CombatContext::new(self, state_tree).run_combat()?;
        Ok(())
    }
}

pub struct CombatContext<'a, 'b> {
    pub integrator: &'a mut Integrator,
    pub state_tree: &'b mut StateTree,
    pub state: ProtectedCell<State>,
    pub current_node: NodeIndex,
}

impl<'a, 'b> CombatContext<'a, 'b> {
    pub fn new(integrator: &'a mut Integrator, state_tree: &'b mut StateTree) -> Self {
        Self {
            state: ProtectedCell::new(integrator.initial_state.clone()),
            current_node: state_tree.root,
            state_tree,
            integrator,
        }
    }

    pub fn run_combat(mut self) -> anyhow::Result<()> {
        self.transition(Transition::BeginCombat)?;

        let mut initiative_rolls = BTreeMap::new();
        for actor in self.state.actors.values() {
            let roll = actor.plan_initiative_roll(RollSettings::default());
            let result = self.integrator.roller.roll(&roll)?;
            initiative_rolls.insert(actor.id, result.total);
        }

        for (actor_id, roll) in &initiative_rolls {
            self.transition(Transition::InitiativeRoll {
                actor: *actor_id,
                roll: *roll,
            })?;
        }

        while self.advance_turn()? {
            // continue advancing turns until combat is over
        }

        self.transition(Transition::EndCombat)?;

        self.integrator.record_combat();
        Ok(())
    }

    pub fn transition(&mut self, transition: Transition) -> anyhow::Result<()> {
        transition.apply(ProtectedCell::get_mut(&mut self.state))?;
        let new_node = self.state_tree.add_node(&self.state);
        self.state_tree
            .add_edge(self.current_node, new_node, transition);
        self.current_node = new_node;

        match transition {
            Transition::BeginCombat => {
                for hook in &mut self.integrator.hooks {
                    hook.on_combat_start(&self.state);
                }
            }
            Transition::BeginTurn { actor } => {
                for hook in &mut self.integrator.hooks {
                    hook.on_turn_start(&self.state, actor, self.state.turn);
                }
            }
            Transition::EndTurn { actor } => {
                for hook in &mut self.integrator.hooks {
                    hook.on_turn_end(&self.state, actor, self.state.turn);
                }
            }
            Transition::EndCombat => {
                for hook in &mut self.integrator.hooks {
                    hook.on_combat_end(&self.state);
                }
            }
            _ => {}
        }

        Ok(())
    }

    fn advance_turn(&mut self) -> anyhow::Result<bool> {
        if self.state.initiative_order.is_empty() {
            return Ok(false);
        }

        if self.state.is_combat_over() {
            return Ok(false);
        }

        // advance to next actor in initiative order
        self.transition(Transition::AdvanceInitiative)?;

        let current_actor_id = self.state.initiative_order[self.state.current_turn_index.unwrap()];

        let Some(current_actor) = self.state.get_actor(current_actor_id) else {
            anyhow::bail!("Current actor not found in simulation state");
        };

        // dead actors skip their turn
        if current_actor.is_unconscious() || current_actor.is_dead() {
            return Ok(true);
        }

        self.transition(Transition::BeginTurn {
            actor: current_actor_id,
        })?;

        for action_type in [ActionEconomyUsage::Action, ActionEconomyUsage::BonusAction] {
            let actor = self
                .state
                .get_actor(current_actor_id)
                .ok_or_else(|| anyhow::anyhow!("Actor not found in simulation state"))?;
            let action_taken = actor.policy.take_action(
                action_type,
                current_actor_id,
                &self.state,
                &mut self.integrator.roller,
            )?;
            self.evaluate_action(current_actor_id, &action_taken)?;

            for hook in &mut self.integrator.hooks {
                hook.on_action_executed(&self.state, &action_taken);
            }
        }

        self.transition(Transition::EndTurn {
            actor: current_actor_id,
        })?;

        Ok(true)
    }

    pub fn evaluate_action(
        &mut self,
        actor_id: ActorId,
        action: &ActionTaken,
    ) -> anyhow::Result<()> {
        if let Some(actor) = self.state.get_actor(actor_id) {
            if actor.is_unconscious() || actor.is_dead() {
                return Ok(());
            }

            if !actor
                .action_economy
                .can_take_action(action.action_economy_usage)
            {
                return Ok(());
            }
        } else {
            anyhow::bail!("Actor not found in simulation state");
        }

        self.transition(Transition::ActionEconomyUsed {
            target: actor_id,
            action_type: action.action_economy_usage,
        })?;

        let Some(actor) = self.state.get_actor(actor_id) else {
            anyhow::bail!("Actor not found in simulation state");
        };

        match &action.action {
            Action::Wait => {}
            Action::UnarmedStrike(UnarmedStrikeAction {
                target,
                attack_roll_settings,
            }) => {
                let target = self
                    .state
                    .actors
                    .get(target)
                    .ok_or_else(|| anyhow::anyhow!("Target actor not found"))?;

                let attack_roll = actor.plan_unarmed_strike_roll(*attack_roll_settings);
                let attack_result = self.integrator.roller.roll(&attack_roll)?;

                let attack_hits = attack_result.meets_dc(target.armor_class as i32);
                let attack_crits = attack_result.is_critical_success();

                if attack_hits {
                    let damage_roll = if attack_crits {
                        actor.plan_unarmed_strike_crit_damage()
                    } else {
                        actor.plan_unarmed_strike_damage()
                    };
                    let damage_result = self.integrator.roller.roll(&damage_roll)?;

                    // apply damage to target
                    // todo: calculate resistances, vulnerabilities, temporary hit points, etc.
                    self.transition(Transition::HealthModification {
                        target: target.id,
                        delta: -damage_result.total,
                    })?;
                }
            }
            Action::Attack(AttackAction {
                weapon_used: weapon_used_id,
                target,
                attack_roll_settings,
            }) => {
                let target = self
                    .state
                    .actors
                    .get(target)
                    .ok_or_else(|| anyhow::anyhow!("Target actor not found"))?;

                let weapon_used = self
                    .state
                    .items
                    .get(weapon_used_id)
                    .ok_or_else(|| anyhow::anyhow!("Weapon item not found"))?;

                let ItemInner::Weapon(weapon_used) = &weapon_used.inner else {
                    return Err(anyhow::anyhow!("Item used for attack is not a weapon"));
                };

                let attack_roll = actor.plan_attack_roll(weapon_used, *attack_roll_settings)?;
                let attack_result = self.integrator.roller.roll(&attack_roll)?;

                let attack_hits = attack_result.meets_dc(target.armor_class as i32);

                if attack_hits {
                    let damage_roll = if attack_result.is_critical_success() {
                        weapon_used
                            .critical_damage
                            .as_ref()
                            .unwrap_or(&weapon_used.damage)
                    } else {
                        &weapon_used.damage
                    };

                    let damage_result = self.integrator.roller.roll(damage_roll)?;

                    // apply damage to target
                    // todo: calculate resistances, vulnerabilities, temporary hit points, etc.
                    self.transition(Transition::HealthModification {
                        target: target.id,
                        delta: -damage_result.total,
                    })?;
                }
            }
            action => todo!("Handle {:?} action", action),
        }

        Ok(())
    }
}
