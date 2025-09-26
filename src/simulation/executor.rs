use std::collections::BTreeMap;

use crate::{
    rules::{actions::ActionEconomyUsage, actor::ActorId, dice::RollSettings},
    simulation::{
        action_evaluator::ActionEvaluator,
        logging::{LogEntry, SimulationLog},
        policy::Policy,
        state::SimulationState,
    },
    statistics::roller::Roller,
};

pub struct SimulationExecutor {
    pub roller: Roller,
    pub state: SimulationState,
    pub log: SimulationLog,
    pub evaluator: ActionEvaluator,
    pub policy: Box<dyn Policy>,
}

impl SimulationExecutor {
    pub fn new(roller: Roller, state: SimulationState, policy: impl Policy) -> Self {
        Self {
            roller,
            state,
            log: SimulationLog::default(),
            evaluator: ActionEvaluator,
            policy: Box::new(policy),
        }
    }

    pub fn save_log(&self, path: &std::path::Path) -> anyhow::Result<()> {
        self.log.save(path)
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        self.begin_combat()?;

        while self.advance_turn()? {}

        self.end_combat()?;

        Ok(())
    }

    pub fn log(&mut self, entry: LogEntry) {
        self.log.log(entry, &self.state);
    }

    pub fn extend_log(&mut self, entries: impl IntoIterator<Item = LogEntry>) {
        self.log.extend(entries, &self.state);
    }

    pub fn begin_combat(&mut self) -> anyhow::Result<()> {
        self.state.turn = 1;
        self.state.current_turn_index = Some(0);

        // ROLL INITIATIVE!!!
        let mut initiatives = BTreeMap::new();
        for actor in self.state.actors.values_mut() {
            let roll = actor.plan_initiative_roll(RollSettings::default());
            let result = roll.roll(&mut self.roller)?;
            actor.initiative = Some(result.total);
            initiatives.insert(actor.id, result);
        }

        for (actor_id, roll) in &initiatives {
            self.log.log(
                LogEntry::InitiativeRoll {
                    actor: *actor_id,
                    roll: roll.clone(),
                },
                &self.state,
            );
        }

        let mut initiatives = self
            .state
            .actors
            .iter()
            .map(|(id, actor)| (*id, actor.initiative.unwrap_or(0)))
            .collect::<Vec<(ActorId, i32)>>();
        initiatives.sort_by(|a, b| b.1.cmp(&a.1));
        initiatives.reverse();
        self.state.initiative_order = initiatives.into_iter().map(|(id, _)| id).collect();
        Ok(())
    }

    pub fn end_combat(&mut self) -> anyhow::Result<()> {
        self.state.turn = 0;
        self.state.current_turn_index = None;
        self.state.initiative_order.clear();
        for actor in self.state.actors.values_mut() {
            actor.initiative = None;
        }
        Ok(())
    }

    pub fn advance_turn(&mut self) -> anyhow::Result<bool> {
        if self.state.initiative_order.is_empty() {
            return Ok(false);
        }

        if self.state.is_combat_over() {
            return Ok(false);
        }

        // advance to next turn
        if let Some(current_index) = self.state.current_turn_index {
            let next_index = (current_index + 1) % self.state.initiative_order.len();
            if next_index == 0 {
                self.state.turn += 1;
            }
            self.state.current_turn_index = Some(next_index);
        } else {
            self.state.current_turn_index = Some(0);
        }

        let current_actor_id = self.state.initiative_order[self.state.current_turn_index.unwrap()];

        let Some(current_actor) = self.state.get_actor(current_actor_id) else {
            anyhow::bail!("Current actor not found in simulation state");
        };

        // dead actors skip their turn
        if current_actor.is_unconscious() || current_actor.is_dead() {
            return Ok(true);
        }

        self.log.log(
            LogEntry::BeginTurn {
                actor: current_actor_id,
            },
            &self.state,
        );

        let current_actor = self.state.get_actor_mut(current_actor_id).unwrap();
        current_actor.action_economy.reset();

        for action_type in [ActionEconomyUsage::Action, ActionEconomyUsage::BonusAction] {
            let action_taken = self.policy.take_action(
                action_type,
                current_actor_id,
                &self.state,
                &mut self.roller,
            )?;
            let action_logs = self.evaluator.evaluate_action(
                current_actor_id,
                &action_taken,
                &self.state,
                &mut self.roller,
            )?;

            for log in &action_logs {
                if let LogEntry::Transition(transition) = log {
                    transition.apply(&mut self.state)?;
                }
            }

            self.log.extend(action_logs, &self.state);
        }

        self.log.log(
            LogEntry::EndTurn {
                actor: current_actor_id,
            },
            &self.state,
        );

        Ok(true)
    }
}
