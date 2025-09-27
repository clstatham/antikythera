use serde::{Deserialize, Serialize};

use crate::rules::dice::RollPlan;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DamageType {
    Bludgeoning,
    Piercing,
    Slashing,
    Fire,
    Cold,
    Lightning,
    Acid,
    Poison,
    Psychic,
    Necrotic,
    Radiant,
    Thunder,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DamageInstance {
    pub roll: RollPlan,
    pub damage_type: DamageType,
}
