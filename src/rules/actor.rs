use crate::{
    rules::{
        death::DeathSaves,
        dice::{RollPlan, RollSettings},
        saves::{SavingThrow, SavingThrowProficiencies},
        skills::{Proficiency, Skill, SkillProficiencies},
        stats::{Stat, StatBlock},
    },
    statistics::roller::Roller,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    pub name: String,
    pub level: u32,
    pub max_health: i32,
    pub health: i32,
    pub armor_class: u32,
    pub stats: StatBlock,
    pub skill_proficiencies: SkillProficiencies,
    pub saving_throw_proficiencies: SavingThrowProficiencies,
    pub death_saves: DeathSaves,
    pub initiative: Option<i32>,
}

impl Actor {
    pub fn is_alive(&self) -> bool {
        self.health > 0
    }

    pub fn is_dead(&self) -> bool {
        self.health <= -self.max_health || self.death_saves.is_dead()
    }

    pub fn heal(&mut self, amount: i32) {
        self.health = (self.health + amount).min(self.max_health);
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.health -= amount;
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

    pub fn roll_death_saving_throw(
        &mut self,
        rng: &mut Roller,
        roll_settings: RollSettings,
    ) -> anyhow::Result<()> {
        if self.is_alive() {
            return Ok(());
        }
        let roll = self.plan_death_saving_throw(roll_settings);
        let result = roll.roll(rng)?;
        if result.meets_dc(10) {
            self.death_saves.record_success();
        } else {
            self.death_saves.record_failure();
        }
        Ok(())
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

    pub fn roll_initiative(
        &mut self,
        rng: &mut Roller,
        roll_settings: RollSettings,
    ) -> anyhow::Result<()> {
        let roll = self.plan_initiative_roll(roll_settings);
        let result = roll.roll(rng)?;
        self.set_initiative(result.total);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_actor() -> Actor {
        Actor {
            name: "Bobby".to_string(),
            level: 1,
            max_health: 100,
            health: 100,
            armor_class: 15,
            stats: StatBlock::default(),
            skill_proficiencies: SkillProficiencies::default(),
            saving_throw_proficiencies: SavingThrowProficiencies::default(),
            death_saves: DeathSaves::default(),
            initiative: None,
        }
    }

    #[test]
    fn test_actor_heal() {
        let mut actor = test_actor();
        actor.take_damage(50);
        assert_eq!(actor.health, 50);
        actor.heal(30);
        assert_eq!(actor.health, 80);
        actor.heal(50);
        assert_eq!(actor.health, 100);
    }

    #[test]
    fn test_actor_take_damage() {
        let mut actor = test_actor();
        actor.take_damage(30);
        assert_eq!(actor.health, 70);
        actor.take_damage(80);
        assert_eq!(actor.health, -10);
    }

    #[test]
    fn test_actor_is_alive() {
        let mut actor = test_actor();
        assert!(actor.is_alive());
        actor.take_damage(100);
        assert!(!actor.is_alive());
    }

    #[test]
    fn test_actor_is_dead() {
        let mut actor = test_actor();
        assert!(!actor.is_dead());
        actor.take_damage(100);
        assert!(!actor.is_dead());
        actor.take_damage(100);
        assert!(actor.is_dead());
    }

    #[test]
    fn test_actor_skill_modifier() {
        let mut actor = test_actor();
        actor.stats.set(Stat::Dexterity, 16); // +3 modifier
        actor
            .skill_proficiencies
            .set(Skill::Acrobatics, Proficiency::Proficient);
        assert_eq!(actor.skill_modifier(Skill::Acrobatics), 4);
        assert_eq!(actor.skill_modifier(Skill::Stealth), 3);
    }

    #[test]
    fn test_actor_saving_throw_modifier() {
        let mut actor = test_actor();
        actor.stats.set(Stat::Constitution, 14); // +2 modifier
        actor
            .saving_throw_proficiencies
            .set(SavingThrow::Constitution, true);
        assert_eq!(actor.saving_throw_modifier(SavingThrow::Constitution), 3);
        assert_eq!(actor.saving_throw_modifier(SavingThrow::Dexterity), 0);
    }

    #[test]
    fn test_actor_plan_skill_check() {
        let mut actor = test_actor();
        actor.stats.set(Stat::Intelligence, 18); // +4 modifier
        actor
            .skill_proficiencies
            .set(Skill::Arcana, Proficiency::Expert);
        let roll = actor.plan_skill_check(Skill::Arcana, RollSettings::default());
        assert_eq!(roll.num_dice, 1);
        assert_eq!(roll.die_size, 20);
        assert_eq!(roll.modifier, 6); // +4 stat mod +2 expert (level 1 * 2)
    }

    #[test]
    fn test_actor_plan_saving_throw() {
        let mut actor = test_actor();
        actor.stats.set(Stat::Wisdom, 12); // +1 modifier
        actor
            .saving_throw_proficiencies
            .set(SavingThrow::Wisdom, true);
        let roll = actor.plan_saving_throw(SavingThrow::Wisdom, RollSettings::default());
        assert_eq!(roll.num_dice, 1);
        assert_eq!(roll.die_size, 20);
        assert_eq!(roll.modifier, 2); // +1 stat mod +1 proficient
    }

    #[test]
    fn test_actor_roll_death_saving_throw() {
        let mut actor = test_actor();
        actor.take_damage(150); // health = -50
        assert!(!actor.is_alive());
        let mut rng = Roller::test_rng();
        for _ in 0..3 {
            actor
                .roll_death_saving_throw(&mut rng, RollSettings::default())
                .unwrap();

            if actor.death_saves.is_dead() {
                break;
            }
            if actor.death_saves.is_stable() {
                break;
            }

            assert!(!actor.is_dead());
        }
        if actor.death_saves.is_dead() {
            assert!(actor.is_dead());
        } else {
            assert!(!actor.is_dead());
        }
    }
}
