use serde::{Deserialize, Serialize};

use crate::{
    rules::{actions::ActionTaken, actor::ActorId, dice::RollResult, items::ItemId},
    simulation::{state::SimulationState, transition::Transition},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogEntry {
    InitiativeRoll {
        actor: ActorId,
        roll: RollResult,
    },
    BeginTurn {
        actor: ActorId,
    },
    EndTurn {
        actor: ActorId,
    },
    Transition(Transition),
    Roll(RollResult),
    Action(ActionTaken),
    AttackAttempt {
        attacker: ActorId,
        target: ActorId,
        weapon: ItemId,
    },
    AttackMiss {
        attacker: ActorId,
        target: ActorId,
        weapon: ItemId,
    },
    AttackHit {
        attacker: ActorId,
        target: ActorId,
        weapon: ItemId,
        damage: i32,
    },
}

impl LogEntry {
    fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &SimulationState,
    ) -> std::fmt::Result {
        match self {
            LogEntry::BeginTurn { actor } => {
                write!(f, "Begin turn for actor ")?;
                actor.pretty_print(f, state)?;
                Ok(())
            }
            LogEntry::EndTurn { actor } => {
                write!(f, "End turn for actor ")?;
                actor.pretty_print(f, state)?;
                Ok(())
            }
            LogEntry::Transition(transition) => write!(f, "Transition: {:?}", transition),
            LogEntry::Roll(roll) => {
                roll.pretty_print(f)?;
                Ok(())
            }
            LogEntry::Action(action) => {
                action.pretty_print(f, state)?;
                Ok(())
            }
            LogEntry::AttackHit {
                attacker,
                target,
                weapon,
                damage,
            } => {
                write!(f, "Actor ")?;
                attacker.pretty_print(f, state)?;
                write!(f, " hits actor ")?;
                target.pretty_print(f, state)?;
                write!(f, " with weapon ")?;
                weapon.pretty_print(f, state)?;
                write!(f, " for {} damage", damage)?;
                Ok(())
            }
            LogEntry::AttackMiss {
                attacker,
                target,
                weapon,
            } => {
                write!(f, "Actor ")?;
                attacker.pretty_print(f, state)?;
                write!(f, " misses actor ")?;
                target.pretty_print(f, state)?;
                write!(f, " with weapon ")?;
                weapon.pretty_print(f, state)?;
                Ok(())
            }

            _ => Ok(()),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimulationLog {
    entries: Vec<LogEntry>,
}

impl SimulationLog {
    pub fn log(&mut self, entry: LogEntry, state: &SimulationState) {
        if matches!(entry, LogEntry::Transition(_)) {
            // Don't log transitions to info, they are too verbose
        } else {
            let mut buf = String::new();
            entry.pretty_print(&mut buf, state).ok();
            if !buf.is_empty() {
                log::info!("{}", buf);
            }
        }

        self.entries.push(entry);
    }

    pub fn extend(&mut self, entries: impl IntoIterator<Item = LogEntry>, state: &SimulationState) {
        for entry in entries {
            self.log(entry, state);
        }
    }
}
