use std::collections::BTreeMap;

use crate::{
    rules::{actions::ActionEconomyUsage, dice::RollSettings},
    simulation::{
        action_evaluator::ActionEvaluator,
        logging::{LogEntry, SimulationLog},
        policy::Policy,
        state::State,
        transition::Transition,
    },
    statistics::roller::Roller,
    utils::ProtectedCell,
};

pub struct Executor {
    pub roller: Roller,
    pub state: ProtectedCell<State>,
    pub log: SimulationLog,
    pub evaluator: ActionEvaluator,
    pub policy: Box<dyn Policy>,
}

impl Executor {
    pub fn new(roller: Roller, state: State, policy: impl Policy) -> Self {
        Self {
            roller,
            state: ProtectedCell::new(state),
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

    pub fn log(&mut self, entry: LogEntry) -> anyhow::Result<()> {
        if let LogEntry::Transition(transition) = &entry {
            transition.apply(ProtectedCell::get_mut(&mut self.state))?;
        }
        self.log.log(entry, &self.state);
        Ok(())
    }

    pub fn extend_log(
        &mut self,
        entries: impl IntoIterator<Item = LogEntry>,
    ) -> anyhow::Result<()> {
        for entry in entries {
            self.log(entry)?;
        }
        Ok(())
    }

    pub fn begin_combat(&mut self) -> anyhow::Result<()> {
        // ROLL INITIATIVE!!!
        let mut initiative_rolls = BTreeMap::new();
        for actor in self.state.actors.values() {
            let roll = actor.plan_initiative_roll(RollSettings::default());
            let result = roll.roll(&mut self.roller)?;
            initiative_rolls.insert(actor.id, result.total);
        }

        for (actor_id, roll) in &initiative_rolls {
            self.log(LogEntry::Transition(Transition::InitiativeRoll {
                actor: *actor_id,
                roll: *roll,
            }))?;
        }

        Ok(())
    }

    pub fn end_combat(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn advance_turn(&mut self) -> anyhow::Result<bool> {
        if self.state.initiative_order.is_empty() {
            return Ok(false);
        }

        if self.state.is_combat_over() {
            return Ok(false);
        }

        // advance to next actor in initiative order
        self.log(LogEntry::Transition(Transition::AdvanceInitiative))?;

        let current_actor_id = self.state.initiative_order[self.state.current_turn_index.unwrap()];

        let Some(current_actor) = self.state.get_actor(current_actor_id) else {
            anyhow::bail!("Current actor not found in simulation state");
        };

        // dead actors skip their turn
        if current_actor.is_unconscious() || current_actor.is_dead() {
            return Ok(true);
        }

        self.log(LogEntry::Transition(Transition::BeginTurn {
            actor: current_actor_id,
        }))?;

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

            self.extend_log(action_logs)?;
        }

        self.log.log(
            LogEntry::Transition(Transition::EndTurn {
                actor: current_actor_id,
            }),
            &self.state,
        );

        Ok(true)
    }
}
