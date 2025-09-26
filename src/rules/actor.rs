use derive_more::{From, Into};
use serde::{Deserialize, Serialize};

use crate::{
    rules::{
        actions::ActionEconomy,
        death::DeathSaves,
        dice::{RollPlan, RollSettings},
        items::{EquippedItems, Inventory, Item},
        saves::{SavingThrow, SavingThrowProficiencies},
        skills::{Proficiency, Skill, SkillProficiencies},
        stats::{Stat, Stats},
    },
    simulation::state::SimulationState,
};

#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, From, Into, Serialize, Deserialize,
)]
pub struct ActorId(pub u32);

impl ActorId {
    pub fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &SimulationState,
    ) -> std::fmt::Result {
        if let Some(actor) = state.actors.get(self) {
            write!(f, "{} (ID: {})", actor.name, self.0)
        } else {
            write!(f, "Unknown Actor (ID: {})", self.0)
        }
    }
}

pub struct ActorBuilder {
    actor: Actor,
}

impl ActorBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            actor: Actor {
                id: ActorId(0), // Placeholder, will be set when added to SimulationState
                name: name.to_string(),
                level: 1,
                max_health: 10,
                health: 10,
                armor_class: 10,
                stats: Stats::default(),
                movement_speed: 30,
                skill_proficiencies: SkillProficiencies::default(),
                saving_throw_proficiencies: SavingThrowProficiencies::default(),
                death_saves: DeathSaves::default(),
                initiative: None,
                action_economy: ActionEconomy::default(),
                equipped_items: EquippedItems::default(),
                inventory: Inventory::default(),
            },
        }
    }

    pub fn level(mut self, level: u32) -> Self {
        self.actor.level = level;
        self
    }

    pub fn max_health(mut self, max_health: i32) -> Self {
        self.actor.max_health = max_health;
        self.actor.health = max_health; // Start at full health
        self
    }

    pub fn armor_class(mut self, armor_class: u32) -> Self {
        self.actor.armor_class = armor_class;
        self
    }

    pub fn stats(mut self, stats: Stats) -> Self {
        self.actor.stats = stats;
        self
    }

    pub fn stat(mut self, stat: Stat, value: u32) -> Self {
        self.actor.stats.set(stat, value);
        self
    }

    pub fn movement_speed(mut self, speed: u32) -> Self {
        self.actor.movement_speed = speed;
        self
    }

    pub fn skill_proficiencies(mut self, proficiencies: SkillProficiencies) -> Self {
        self.actor.skill_proficiencies = proficiencies;
        self
    }

    pub fn skill_proficiency(mut self, skill: Skill, proficiency: Proficiency) -> Self {
        self.actor.skill_proficiencies.set(skill, proficiency);
        self
    }

    pub fn saving_throw_proficiencies(mut self, proficiencies: SavingThrowProficiencies) -> Self {
        self.actor.saving_throw_proficiencies = proficiencies;
        self
    }

    pub fn saving_throw_proficiency(mut self, save: SavingThrow, proficient: bool) -> Self {
        self.actor.saving_throw_proficiencies.set(save, proficient);
        self
    }

    pub fn build(self) -> Actor {
        self.actor
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub id: ActorId,
    pub name: String,
    pub level: u32,
    pub max_health: i32,
    pub health: i32,
    pub armor_class: u32,
    pub stats: Stats,
    pub movement_speed: u32,
    pub skill_proficiencies: SkillProficiencies,
    pub saving_throw_proficiencies: SavingThrowProficiencies,
    pub death_saves: DeathSaves,
    pub initiative: Option<i32>,
    pub action_economy: ActionEconomy,
    pub equipped_items: EquippedItems,
    pub inventory: Inventory,
}

impl Actor {
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn is_unconscious(&self) -> bool {
        self.health <= 0 && !self.is_dead()
    }

    pub fn is_dead(&self) -> bool {
        self.health <= -self.max_health || self.death_saves.is_dead()
    }

    pub fn skill_modifier(&self, skill: Skill) -> i32 {
        let stat = skill.associated_stat();
        let stat_mod = self.stats.modifier(stat);
        let proficiency = self.skill_proficiencies.get(skill);
        let proficiency_bonus = match proficiency {
            Proficiency::None => 0,
            Proficiency::HalfProficient => self.level / 2,
            Proficiency::Proficient => self.level,
            Proficiency::Expert => self.level * 2,
        };
        stat_mod + proficiency_bonus as i32
    }

    pub fn stat_modifier(&self, stat: Stat) -> i32 {
        self.stats.modifier(stat)
    }

    pub fn saving_throw_modifier(&self, save: SavingThrow) -> i32 {
        let associated_stat = save.to_stat();
        let stat_mod = self.stats.modifier(associated_stat);
        let is_proficient = self.saving_throw_proficiencies.get(save);
        let proficiency_bonus = if is_proficient { self.level } else { 0 };
        stat_mod + proficiency_bonus as i32
    }

    pub fn plan_unarmed_strike_roll(&self, roll_settings: RollSettings) -> RollPlan {
        let attack_modifier = self.stat_modifier(Stat::Strength);
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier: attack_modifier,
            settings: roll_settings,
        }
    }

    pub fn plan_unarmed_strike_damage(&self) -> RollPlan {
        let damage_modifier = self.stat_modifier(Stat::Strength);
        RollPlan {
            num_dice: 1,
            die_size: 4,
            modifier: damage_modifier,
            settings: RollSettings::default(),
        }
    }

    pub fn plan_unarmed_strike_crit_damage(&self) -> RollPlan {
        let damage_modifier = self.stat_modifier(Stat::Strength);
        RollPlan {
            num_dice: 2,
            die_size: 4,
            modifier: damage_modifier,
            settings: RollSettings::default(),
        }
    }

    pub fn plan_skill_check(&self, skill: Skill, roll_settings: RollSettings) -> RollPlan {
        let modifier = self.skill_modifier(skill);
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier,
            settings: roll_settings,
        }
    }

    pub fn plan_saving_throw(&self, save: SavingThrow, roll_settings: RollSettings) -> RollPlan {
        let modifier = self.saving_throw_modifier(save);
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier,
            settings: roll_settings,
        }
    }

    pub fn plan_death_saving_throw(&self, roll_settings: RollSettings) -> RollPlan {
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier: 0,
            settings: roll_settings,
        }
    }

    pub fn plan_initiative_roll(&self, roll_settings: RollSettings) -> RollPlan {
        let dex_mod = self.stats.modifier(Stat::Dexterity);
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier: dex_mod,
            settings: roll_settings,
        }
    }

    pub fn set_initiative(&mut self, initiative: i32) {
        self.initiative = Some(initiative);
    }

    pub fn give_item(&mut self, item: Item, quantity: u32) {
        self.inventory.add_item(item, quantity);
    }

    #[cfg(test)]
    pub fn test_actor(id: u32, name: &str) -> Self {
        Self {
            id: ActorId(id),
            name: name.to_string(),
            level: 1,
            max_health: 10,
            health: 10,
            armor_class: 15,
            stats: Stats::default(),
            movement_speed: 30,
            skill_proficiencies: SkillProficiencies::default(),
            saving_throw_proficiencies: SavingThrowProficiencies::default(),
            death_saves: DeathSaves::default(),
            initiative: None,
            action_economy: ActionEconomy::default(),
            equipped_items: EquippedItems::default(),
            inventory: Inventory::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_actor_is_alive() {
        let actor = Actor::test_actor(1, "Test Actor");
        assert!(actor.is_alive());
        assert!(!actor.is_dead());
    }
}
