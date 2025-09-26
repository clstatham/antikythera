use serde::{Deserialize, Serialize};

use crate::{
    rules::{actions::ActionType, actor::ActorId, stats::Stat},
    simulation::state::SimulationState,
};

/// A transition represents a ***single***, atomic change from one simulation state to another.
/// For instance, it could represent a single amount of damage being dealt, or a stat modifier being applied or removed.
///
/// Transitions can be generated as a result of actions or rolls, and every transition is logged in the simulation log.
///
/// Transitions can be thought of as "operations" to be applied to the simulation state, and they should be
/// the only mechanism by which the simulation state is modified externally.
///
/// Transitions should be deterministic and side-effect free.
/// This means that transitions should not contain any random elements or references to external state.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Transition {
    HealthModification {
        target: ActorId,
        delta: i32, // positive for healing, negative for damage
    },
    StatModification {
        target: ActorId,
        stat: Stat,
        delta: i32,
    },
    ActionUsed {
        target: ActorId,
        action_type: ActionType,
    },
    ActionReset {
        target: ActorId,
    },
}

impl Transition {
    pub fn apply(&self, state: &mut SimulationState) -> anyhow::Result<()> {
        match self {
            Transition::HealthModification { target, delta } => {
                if let Some(actor) = state.actors.get_mut(target) {
                    actor.health += *delta;
                }
            }
            Transition::StatModification {
                target,
                stat,
                delta,
            } => {
                if let Some(actor) = state.actors.get_mut(target) {
                    *actor.stats.get_mut(*stat) += *delta as u32;
                }
            }
            Transition::ActionUsed {
                target,
                action_type,
            } => {
                if let Some(actor) = state.actors.get_mut(target) {
                    actor.action_economy.use_action(*action_type)?;
                }
            }
            Transition::ActionReset { target } => {
                if let Some(actor) = state.actors.get_mut(target) {
                    actor.action_economy.reset();
                }
            }
        }

        Ok(())
    }
}
