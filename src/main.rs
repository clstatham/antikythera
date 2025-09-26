use crate::{
    rules::{
        actor::ActorBuilder,
        items::{ItemType, WeaponBuilder, WeaponProficiency, WeaponType},
        saves::SavingThrow,
        skills::{Skill, SkillProficiency},
        stats::Stat,
    },
    simulation::{executor::Executor, policy::RandomPolicy, state::State},
    statistics::roller::Roller,
};

pub mod roll_parser;
pub mod rules;
pub mod simulation;
pub mod statistics;
pub mod utils;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "[{}] - {}", record.level(), record.args())
        })
        .filter_level(log::LevelFilter::Debug)
        .try_init()?;

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
        .armor_class(7)
        .max_health(15)
        .level(1)
        .build();

    goblin.give_item(sword.clone(), 1);

    let hero = state.add_actor(hero);
    let goblin = state.add_actor(goblin);
    state.add_ally_group(vec![hero]);
    state.add_ally_group(vec![goblin]);

    let roller = Roller::new();
    let policy = RandomPolicy;
    let mut executor = Executor::new(roller, state, policy);
    executor.run()?;

    executor.save_log(std::path::Path::new("target/simulation_log.json"))?;

    Ok(())
}
