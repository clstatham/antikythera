use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

use crate::{
    rules::{actor::ActorId, damage::DamageInstance, dice::RollPlan, items::ItemId, stats::Stat},
    simulation::state::State,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, From, Into)]
pub struct SpellId(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellEffect {
    SpellAttack {
        to_hit: RollPlan,
        damage: Vec<DamageInstance>,
    },
    Damage {
        damage: Vec<DamageInstance>,
    },
    Heal {
        amount: RollPlan,
    },
    Buff {
        stat: Stat,
        amount: i32,
        duration_rounds: u32,
    },
    Debuff {
        stat: Stat,
        amount: i32,
        duration_rounds: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpellComponents {
    pub verbal: bool,
    pub somatic: bool,
    pub material: Option<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpellTargetType {
    SelfTarget,
    Ally,
    Enemy,
    Area,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spell {
    pub id: SpellId,
    pub name: String,
    pub level: u8,
    pub casting_time: String,
    pub range: String,
    pub components: SpellComponents,
    pub duration_rounds: Option<u32>,
    pub target_types: Vec<SpellTargetType>,
    pub effects: Vec<SpellEffect>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SpellTarget {
    SelfTarget,
    Ally(ActorId),
    Enemy(ActorId),
    Area { x: f32, y: f32, radius: f32 }, // todo: support shapes (cone, line, etc.)
}

impl SpellTarget {
    pub fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &State,
    ) -> std::fmt::Result {
        match self {
            SpellTarget::SelfTarget => write!(f, "themself"),
            SpellTarget::Ally(actor_id) | SpellTarget::Enemy(actor_id) => {
                actor_id.pretty_print(f, state)
            }
            SpellTarget::Area { x, y, radius } => {
                write!(f, "Area at ({}, {}) with radius {}", x, y, radius)
            }
        }
    }
}
