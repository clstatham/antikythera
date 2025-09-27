pub mod roll_parser;
pub mod rules;
pub mod simulation;
pub mod statistics;
pub mod utils;

#[cfg(test)]
mod tests {
    use crate::{
        rules::{
            actor::ActorBuilder,
            items::{ItemType, WeaponBuilder, WeaponProficiency, WeaponType},
            saves::SavingThrow,
            skills::{Skill, SkillProficiency},
            stats::Stat,
        },
        simulation::state::State,
        statistics::{
            integration::Integrator,
            query::{OutcomeConditionProbability, Query},
            roller::Roller,
        },
    };

    #[test]
    fn test_demo() -> anyhow::Result<()> {
        let mut state = State::new();

        let sword = WeaponBuilder::new(WeaponType::Longsword)
            .attack_bonus(1)
            .damage("1d8+3")
            .critical_damage("2d8+3")
            .build();

        let sword = state.add_item("Longsword", ItemType::Weapon(sword));

        let mut hero = ActorBuilder::new("Hero")
            .stat(Stat::Strength, 16)
            .stat(Stat::Dexterity, 14)
            .stat(Stat::Constitution, 14)
            .stat(Stat::Intelligence, 10)
            .stat(Stat::Wisdom, 12)
            .stat(Stat::Charisma, 10)
            .movement_speed(30)
            .skill_proficiency(Skill::Athletics, SkillProficiency::Proficient)
            .skill_proficiency(Skill::Perception, SkillProficiency::HalfProficient)
            .saving_throw_proficiency(SavingThrow::Strength, true)
            .saving_throw_proficiency(SavingThrow::Constitution, true)
            .weapon_proficiency(WeaponType::Longsword, WeaponProficiency::Proficient)
            .armor_class(16)
            .max_health(30)
            .level(3)
            .build();

        hero.give_item(sword.clone(), 1);

        let mut goblin = ActorBuilder::new("Goblin")
            .stat(Stat::Strength, 8)
            .stat(Stat::Dexterity, 14)
            .stat(Stat::Constitution, 10)
            .stat(Stat::Intelligence, 10)
            .stat(Stat::Wisdom, 8)
            .stat(Stat::Charisma, 8)
            .movement_speed(30)
            .skill_proficiency(Skill::Stealth, SkillProficiency::Proficient)
            .saving_throw_proficiency(SavingThrow::Dexterity, true)
            .armor_class(15)
            .max_health(13)
            .level(1)
            .build();

        goblin.give_item(sword.clone(), 1);

        let goblin2 = goblin.clone();

        let hero = state.add_actor(hero);
        let goblin = state.add_actor(goblin);
        let goblin2 = state.add_actor(goblin2);
        state.add_ally_group(vec![hero]);
        state.add_ally_group(vec![goblin, goblin2]);

        let roller = Roller::new();
        let mut integrator = Integrator::new(100, roller, state);
        integrator.run()?;

        let stats = integrator.compute_statistics();
        stats.print_summary();

        let query = OutcomeConditionProbability::new(move |state: &State| {
            state.get_actor(hero).map(|a| a.is_alive()).unwrap()
        });
        let prob = query.query(integrator.state_tree(), &stats)?;
        println!("Probability that hero is alive: {:.2}%", prob * 100.0);
        let query = OutcomeConditionProbability::new(move |state: &State| {
            state.get_actor(goblin).map(|a| a.is_alive()).unwrap()
        });
        let prob = query.query(integrator.state_tree(), &stats)?;
        println!("Probability that goblin 1 is alive: {:.2}%", prob * 100.0);
        let query = OutcomeConditionProbability::new(move |state: &State| {
            state.get_actor(goblin2).map(|a| a.is_alive()).unwrap()
        });
        let prob = query.query(integrator.state_tree(), &stats)?;
        println!("Probability that goblin 2 is alive: {:.2}%", prob * 100.0);

        Ok(())
    }
}
