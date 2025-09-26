use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::rules::{
    dice::{RollPlan, RollSettings},
    stats::Stat,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skill {
    Acrobatics,
    AnimalHandling,
    Arcana,
    Athletics,
    Deception,
    History,
    Insight,
    Intimidation,
    Investigation,
    Medicine,
    Nature,
    Perception,
    Performance,
    Persuasion,
    Religion,
    SleightOfHand,
    Stealth,
    Survival,
}

impl Skill {
    pub fn all() -> Vec<Skill> {
        vec![
            Skill::Acrobatics,
            Skill::AnimalHandling,
            Skill::Arcana,
            Skill::Athletics,
            Skill::Deception,
            Skill::History,
            Skill::Insight,
            Skill::Intimidation,
            Skill::Investigation,
            Skill::Medicine,
            Skill::Nature,
            Skill::Perception,
            Skill::Performance,
            Skill::Persuasion,
            Skill::Religion,
            Skill::SleightOfHand,
            Skill::Stealth,
            Skill::Survival,
        ]
    }

    pub fn associated_stat(&self) -> Stat {
        match self {
            Skill::Acrobatics => Stat::Dexterity,
            Skill::AnimalHandling => Stat::Wisdom,
            Skill::Arcana => Stat::Intelligence,
            Skill::Athletics => Stat::Strength,
            Skill::Deception => Stat::Charisma,
            Skill::History => Stat::Intelligence,
            Skill::Insight => Stat::Wisdom,
            Skill::Intimidation => Stat::Charisma,
            Skill::Investigation => Stat::Intelligence,
            Skill::Medicine => Stat::Wisdom,
            Skill::Nature => Stat::Intelligence,
            Skill::Perception => Stat::Wisdom,
            Skill::Performance => Stat::Charisma,
            Skill::Persuasion => Stat::Charisma,
            Skill::Religion => Stat::Intelligence,
            Skill::SleightOfHand => Stat::Dexterity,
            Skill::Stealth => Stat::Dexterity,
            Skill::Survival => Stat::Wisdom,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Proficiency {
    None,
    HalfProficient, // e.g., for Jack of All Trades (Bard feature)
    Proficient,
    Expert,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillProficiencies {
    proficiencies: FxHashMap<Skill, Proficiency>,
}

impl Default for SkillProficiencies {
    fn default() -> Self {
        let mut proficiencies = FxHashMap::default();
        for skill in Skill::all() {
            proficiencies.insert(skill, Proficiency::None);
        }
        SkillProficiencies { proficiencies }
    }
}

impl SkillProficiencies {
    pub fn with_proficiency(mut self, skill: Skill, proficiency: Proficiency) -> Self {
        self.proficiencies.insert(skill, proficiency);
        self
    }

    pub fn get(&self, skill: Skill) -> Proficiency {
        *self.proficiencies.get(&skill).unwrap_or(&Proficiency::None)
    }

    pub fn set(&mut self, skill: Skill, proficiency: Proficiency) {
        self.proficiencies.insert(skill, proficiency);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillCheck {
    pub skill: Skill,
    pub proficiency: Proficiency,
    pub proficiency_bonus: u32,
    pub modifier: i32,
    pub roll_settings: RollSettings,
}

impl SkillCheck {
    pub fn total_modifier(&self) -> i32 {
        let proficiency_bonus = match self.proficiency {
            Proficiency::None => 0,
            Proficiency::HalfProficient => self.proficiency_bonus / 2,
            Proficiency::Proficient => self.proficiency_bonus,
            Proficiency::Expert => self.proficiency_bonus * 2,
        };
        self.modifier + proficiency_bonus as i32
    }

    pub fn roll(&self) -> RollPlan {
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier: self.total_modifier(),
            settings: self.roll_settings,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::dice::Advantage;

    use super::*;

    #[test]
    fn test_skill_check_total_modifier() {
        let check = SkillCheck {
            skill: Skill::Stealth,
            proficiency: Proficiency::Proficient,
            proficiency_bonus: 4,
            modifier: 1,
            roll_settings: RollSettings {
                advantage: Advantage::Normal,
                minimum_die_value: None,
                maximum_die_value: None,
                reroll_dice_below: None,
            },
        };
        assert_eq!(check.total_modifier(), 5);
    }

    #[test]
    fn test_skill_check_roll() {
        let check = SkillCheck {
            skill: Skill::Stealth,
            proficiency: Proficiency::Expert,
            proficiency_bonus: 4,
            modifier: 1,
            roll_settings: RollSettings {
                advantage: Advantage::Advantage,
                minimum_die_value: None,
                maximum_die_value: None,
                reroll_dice_below: None,
            },
        };
        let roll = check.roll();
        assert_eq!(roll.num_dice, 1);
        assert_eq!(roll.die_size, 20);
        assert_eq!(roll.modifier, 9); // 1 + (2 * 4)
        assert_eq!(roll.settings.advantage, Advantage::Advantage);
    }
}
