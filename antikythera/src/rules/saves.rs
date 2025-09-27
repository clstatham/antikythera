use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::rules::stats::Stat;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SavingThrow {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl SavingThrow {
    pub fn all() -> Vec<SavingThrow> {
        vec![
            SavingThrow::Strength,
            SavingThrow::Dexterity,
            SavingThrow::Constitution,
            SavingThrow::Intelligence,
            SavingThrow::Wisdom,
            SavingThrow::Charisma,
        ]
    }

    pub fn to_stat(&self) -> Stat {
        match self {
            SavingThrow::Strength => Stat::Strength,
            SavingThrow::Dexterity => Stat::Dexterity,
            SavingThrow::Constitution => Stat::Constitution,
            SavingThrow::Intelligence => Stat::Intelligence,
            SavingThrow::Wisdom => Stat::Wisdom,
            SavingThrow::Charisma => Stat::Charisma,
        }
    }

    pub fn from_stat(stat: Stat) -> SavingThrow {
        match stat {
            Stat::Strength => SavingThrow::Strength,
            Stat::Dexterity => SavingThrow::Dexterity,
            Stat::Constitution => SavingThrow::Constitution,
            Stat::Intelligence => SavingThrow::Intelligence,
            Stat::Wisdom => SavingThrow::Wisdom,
            Stat::Charisma => SavingThrow::Charisma,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct SavingThrowProficiencies {
    save_proficiencies: BTreeMap<SavingThrow, bool>,
}

impl SavingThrowProficiencies {
    pub fn with_proficiency(mut self, save: SavingThrow, proficient: bool) -> Self {
        self.set(save, proficient);
        self
    }

    pub fn set(&mut self, save: SavingThrow, proficient: bool) {
        self.save_proficiencies.insert(save, proficient);
    }

    pub fn get(&self, save: SavingThrow) -> bool {
        *self.save_proficiencies.get(&save).unwrap_or(&false)
    }
}
