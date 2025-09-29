use serde::{Deserialize, Serialize};

use crate::{
    rules::{actions::ActionEconomyUsage, actor::ActorId, stats::Stat},
    simulation::state::State,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransitionType {
    Root,
    BeginCombat,
    EndCombat,
    InitiativeRoll,
    BeginTurn,
    EndTurn,
    AdvanceInitiative,
    HealthModification,
    StatModification,
    ActionEconomyUsed,
}

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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum Transition {
    Root,
    BeginCombat,
    EndCombat,
    InitiativeRoll {
        actor: ActorId,
        roll: i32,
    },
    BeginTurn {
        actor: ActorId,
    },
    EndTurn {
        actor: ActorId,
    },
    AdvanceInitiative,
    HealthModification {
        target: ActorId,
        delta: i32, // positive for healing, negative for damage
    },
    StatModification {
        target: ActorId,
        stat: Stat,
        delta: i32,
    },
    ActionEconomyUsed {
        target: ActorId,
        action_type: ActionEconomyUsage,
    },
}

impl Transition {
    pub fn transition_type(&self) -> TransitionType {
        match self {
            Transition::Root => TransitionType::Root,
            Transition::BeginCombat => TransitionType::BeginCombat,
            Transition::EndCombat => TransitionType::EndCombat,
            Transition::InitiativeRoll { .. } => TransitionType::InitiativeRoll,
            Transition::BeginTurn { .. } => TransitionType::BeginTurn,
            Transition::EndTurn { .. } => TransitionType::EndTurn,
            Transition::AdvanceInitiative => TransitionType::AdvanceInitiative,
            Transition::HealthModification { .. } => TransitionType::HealthModification,
            Transition::StatModification { .. } => TransitionType::StatModification,
            Transition::ActionEconomyUsed { .. } => TransitionType::ActionEconomyUsed,
        }
    }

    pub fn emoji(&self) -> &'static str {
        match self {
            Transition::Root => "root",
            Transition::ActionEconomyUsed { .. } => "⚔️",
            Transition::BeginCombat => "🎬",
            Transition::EndCombat => "🏁",
            Transition::InitiativeRoll { .. } => "🎲",
            Transition::BeginTurn { .. } => "▶️",
            Transition::EndTurn { .. } => "⏸️",
            Transition::AdvanceInitiative => "➡️",
            Transition::HealthModification { delta, .. } => {
                if *delta >= 0 {
                    "💚"
                } else {
                    "💔"
                }
            }
            Transition::StatModification { delta, .. } => {
                if *delta >= 0 {
                    "📈"
                } else {
                    "📉"
                }
            }
        }
    }

    #[allow(clippy::match_like_matches_macro)]
    pub fn is_quiet(&self) -> bool {
        match self {
            Transition::ActionEconomyUsed { .. } => true,
            Transition::AdvanceInitiative => true,
            _ => false,
        }
    }

    pub fn apply(&self, state: &mut State) -> anyhow::Result<()> {
        match self {
            Transition::Root => {}
            Transition::BeginCombat => {
                state.current_turn_index = Some(0);
            }
            Transition::EndCombat => {
                state.current_turn_index = None;

                state.turn = 0;
                state.current_turn_index = None;
                state.initiative_order.clear();
                for actor in state.actors.values_mut() {
                    actor.initiative = None;
                }
            }
            Transition::InitiativeRoll { actor, roll } => {
                if let Some(actor) = state.actors.get_mut(actor) {
                    actor.initiative = Some(*roll);
                }

                // recalculate initiative order
                let mut initiatives = state
                    .actors
                    .iter()
                    .map(|(id, actor)| (*id, actor.initiative.unwrap_or(0)))
                    .collect::<Vec<(ActorId, i32)>>();
                initiatives.sort_by(|a, b| b.1.cmp(&a.1)); // descending order
                state.initiative_order = initiatives.into_iter().map(|(id, _)| id).collect();
            }
            Transition::BeginTurn { actor } => {
                if let Some(actor) = state.actors.get_mut(actor) {
                    actor.action_economy.reset();
                }
            }
            Transition::EndTurn { actor: _ } => {}
            Transition::AdvanceInitiative => {
                if let Some(current_index) = state.current_turn_index {
                    let next_index = (current_index + 1) % state.initiative_order.len();
                    if next_index == 0 {
                        // top of the round
                        state.turn += 1;
                    }
                    state.current_turn_index = Some(next_index);
                } else {
                    state.current_turn_index = Some(0);
                }
            }
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
            Transition::ActionEconomyUsed {
                target,
                action_type,
            } => {
                if let Some(actor) = state.actors.get_mut(target) {
                    actor.action_economy.use_action(*action_type)?;
                }
            }
        }

        Ok(())
    }

    pub fn pretty_print(&self, f: &mut impl std::fmt::Write, state: &State) -> std::fmt::Result {
        match self {
            Transition::Root => write!(f, "<Initial State>"),
            Transition::InitiativeRoll { actor, roll } => {
                actor.pretty_print(f, state)?;
                write!(f, " rolls initiative: {}", roll)
            }
            Transition::BeginCombat => write!(f, "Begin Combat"),
            Transition::EndCombat => write!(f, "End Combat"),
            Transition::AdvanceInitiative => write!(f, "Advance Initiative"),
            Transition::BeginTurn { actor } => {
                actor.pretty_print(f, state)?;
                write!(f, " begins their turn")
            }
            Transition::EndTurn { actor } => {
                actor.pretty_print(f, state)?;
                write!(f, " ends their turn")
            }
            Transition::HealthModification { target, delta } => {
                target.pretty_print(f, state)?;
                write!(f, " takes {}", delta.abs())?;
                if *delta >= 0 {
                    write!(f, " healing")
                } else {
                    write!(f, " damage")
                }
            }
            Transition::StatModification {
                target,
                stat,
                delta,
            } => {
                target.pretty_print(f, state)?;
                write!(f, "'s {:?} is ", stat)?;
                if *delta >= 0 {
                    write!(f, "increased by {}", delta)
                } else {
                    write!(f, "decreased by {}", delta.abs())
                }
            }
            Transition::ActionEconomyUsed {
                action_type,
                target,
            } => {
                target.pretty_print(f, state)?;
                write!(f, " uses their {:?}", action_type)
            }
        }
    }
}
