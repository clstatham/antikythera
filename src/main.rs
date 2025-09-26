use crate::{
    rules::{
        actor::ActorBuilder,
        items::{ItemType, WeaponBuilder},
        saves::SavingThrow,
        skills::{Proficiency, Skill},
        stats::Stat,
    },
    simulation::{executor::SimulationExecutor, policy::RandomPolicy, state::SimulationState},
    statistics::roller::Roller,
};

pub mod roll_parser;
pub mod rules;
pub mod simulation;
pub mod statistics;

fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .format(|buf, record| {
            use std::io::Write;
            writeln!(buf, "[{}] - {}", record.level(), record.args())
        })
        .filter_level(log::LevelFilter::Debug)
        .try_init()?;

    let mut state = SimulationState::new();

    let sword = WeaponBuilder::new()
        .attack_bonus(5)
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
        .skill_proficiency(Skill::Athletics, Proficiency::Proficient)
        .skill_proficiency(Skill::Perception, Proficiency::HalfProficient)
        .saving_throw_proficiency(SavingThrow::Strength, true)
        .saving_throw_proficiency(SavingThrow::Constitution, true)
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
        .skill_proficiency(Skill::Stealth, Proficiency::Proficient)
        .saving_throw_proficiency(SavingThrow::Dexterity, true)
        .armor_class(15)
        .max_health(7)
        .level(1)
        .build();

    goblin.give_item(sword.clone(), 1);

    let hero = state.add_actor(hero);
    let goblin = state.add_actor(goblin);
    state.add_ally_group(vec![hero]);
    state.add_ally_group(vec![goblin]);

    let roller = Roller::new();
    let policy = RandomPolicy;
    let mut executor = SimulationExecutor::new(roller, state, policy);
    executor.run()?;

    executor.save_log(std::path::Path::new("target/simulation_log.json"))?;

    Ok(())
}
