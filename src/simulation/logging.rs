use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthStr;

use crate::{
    rules::{
        actions::{Action, ActionTaken},
        actor::ActorId,
        dice::RollResult,
        items::ItemId,
    },
    simulation::{state::State, transition::Transition},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogEntry {
    Transition(Transition),
    Roll(RollResult),
    Action(ActionTaken),
    AttackMiss {
        attacker: ActorId,
        target: ActorId,
        weapon: ItemId,
    },
    AttackHit {
        attacker: ActorId,
        target: ActorId,
        weapon: ItemId,
    },
    ActorDowned {
        actor: ActorId,
    },
    ActorStabilized {
        actor: ActorId,
    },
    ActorKilled {
        actor: ActorId,
    },
}

impl LogEntry {
    pub fn emoji(&self) -> &'static str {
        match self {
            LogEntry::Transition(t) => t.emoji(),
            LogEntry::Roll(_) => "🎲",
            LogEntry::Action(a) => match a.action {
                Action::Wait => "💤",
                _ => "⚔️",
            },
            LogEntry::AttackHit { .. } => "💥",
            LogEntry::AttackMiss { .. } => "❌",
            LogEntry::ActorDowned { .. } => "💀",
            LogEntry::ActorStabilized { .. } => "❤️‍🩹",
            LogEntry::ActorKilled { .. } => "☠️",
        }
    }

    pub fn is_quiet(&self) -> bool {
        match self {
            LogEntry::Transition(t) => t.is_quiet(),
            LogEntry::Action(a) => matches!(a.action, Action::Wait),
            _ => false,
        }
    }

    fn pretty_print(&self, f: &mut impl std::fmt::Write, state: &State) -> std::fmt::Result {
        match self {
            LogEntry::Transition(transition) => transition.pretty_print(f, state),
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
            } => {
                attacker.pretty_print(f, state)?;
                write!(f, " hits ")?;
                target.pretty_print(f, state)?;
                write!(f, " with their ")?;
                weapon.pretty_print(f, state)?;
                Ok(())
            }
            LogEntry::AttackMiss {
                attacker,
                target,
                weapon,
            } => {
                attacker.pretty_print(f, state)?;
                write!(f, " misses  ")?;
                target.pretty_print(f, state)?;
                write!(f, " with their ")?;
                weapon.pretty_print(f, state)?;
                Ok(())
            }
            LogEntry::ActorDowned { actor } => {
                actor.pretty_print(f, state)?;
                write!(f, " is downed")?;
                Ok(())
            }
            LogEntry::ActorStabilized { actor } => {
                actor.pretty_print(f, state)?;
                write!(f, " is stabilized")?;
                Ok(())
            }
            LogEntry::ActorKilled { actor } => {
                actor.pretty_print(f, state)?;
                write!(f, " is killed")?;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SimulationLog {
    entries: Vec<LogEntry>,
}

impl SimulationLog {
    pub fn log(&mut self, entry: LogEntry, state: &State) {
        if !entry.is_quiet() {
            let mut buf = String::new();

            let emoji = entry.emoji();
            let emoji = format_emoji(emoji, 2);
            buf.push_str(&emoji);
            buf.push(' ');

            entry.pretty_print(&mut buf, state).ok();
            log::info!("{}", buf);
        }

        self.entries.push(entry);
    }

    pub fn save(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let file = std::fs::File::create(path)?;
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }
}

fn emoji_emoji_presentation(s: &str) -> String {
    if s.chars().any(|c| c == '\u{FE0F}' || c == '\u{200D}') {
        s.to_string()
    } else {
        format!("{s}\u{FE0F}")
    }
}

fn pad_cells(s: &str, field_cells: usize) -> String {
    let w = s.width();
    let pad = field_cells.saturating_sub(w);
    format!("{s}{}", " ".repeat(pad))
}

fn format_emoji(emoji: &str, field_cells: usize) -> String {
    let e = emoji_emoji_presentation(emoji);
    pad_cells(&e, field_cells)
}
