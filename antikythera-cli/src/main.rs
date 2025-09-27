use std::path::PathBuf;

use antikythera::{prelude::*, statistics::state_tree::StateTreeStats};
use clap::Parser;
use serde::Serialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Initial state JSON file path
    #[arg(short, long, default_value = "state.json", value_name = "FILE")]
    state: PathBuf,

    /// Use a demo state instead of loading from file (for testing)
    #[arg(long, default_value_t = false, conflicts_with = "state")]
    demo: bool,

    /// Number of combats to simulate
    #[arg(short, long, default_value_t = 1000)]
    combats: usize,

    /// Random seed for reproducibility
    #[arg(long, default_value = None)]
    seed: Option<u64>,

    /// Output file path
    #[arg(short, long, default_value = "antikythera-statistics.json")]
    output: PathBuf,
}

#[derive(Debug, Serialize)]
pub struct Statistics {
    pub initial_state: State,
    pub total_combats: usize,
    pub state_tree: StateTree,
    pub state_tree_stats: StateTreeStats,
}

pub fn demo_state() -> State {
    let mut state = State::new();

    let sword = WeaponBuilder::new(WeaponType::Longsword)
        .attack_bonus(1)
        .damage("1d8+3")
        .critical_damage("2d8+3")
        .build();

    let sword = state.add_item("Longsword", ItemInner::Weapon(sword));

    let mut hero = ActorBuilder::new("Hero")
        .group(0)
        .stat(Stat::Strength, 16)
        .stat(Stat::Dexterity, 12)
        .stat(Stat::Constitution, 14)
        .stat(Stat::Intelligence, 10)
        .stat(Stat::Wisdom, 10)
        .stat(Stat::Charisma, 10)
        .skill_proficiency(Skill::Athletics, SkillProficiency::Proficient)
        .skill_proficiency(Skill::Perception, SkillProficiency::Proficient)
        .saving_throw_proficiency(SavingThrow::Strength, true)
        .saving_throw_proficiency(SavingThrow::Constitution, true)
        .max_health(12)
        .level(1) // 10 + 3 (Chain Mail) + 2 (Shield) + 0 (Dex)
        .weapon_proficiency(WeaponType::Longsword, WeaponProficiency::Proficient)
        .build();

    hero.give_item(sword, 1);

    let mut goblin1 = ActorBuilder::new("Goblin")
        .group(1)
        .stat(Stat::Strength, 8)
        .stat(Stat::Dexterity, 14)
        .stat(Stat::Constitution, 10)
        .stat(Stat::Intelligence, 10)
        .stat(Stat::Wisdom, 8)
        .stat(Stat::Charisma, 8)
        .skill_proficiency(Skill::Stealth, SkillProficiency::Proficient)
        .saving_throw_proficiency(SavingThrow::Dexterity, true)
        .max_health(7)
        .level(1)
        .build();

    let mut goblin2 = goblin1.clone();

    goblin1.give_item(sword, 1);
    goblin2.give_item(sword, 1);

    let _hero = state.add_actor(hero);
    let _goblin1 = state.add_actor(goblin1);
    let _goblin2 = state.add_actor(goblin2);

    state
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    env_logger::builder()
        .format_timestamp_secs()
        .filter_level(log::LevelFilter::Info)
        .init();
    log::info!("Starting simulation with args: {:?}", args);

    let roller = match args.seed {
        Some(seed) => Roller::from_seed(seed),
        None => Roller::new(),
    };
    let initial_state = if args.demo {
        log::info!("Using demo state");
        demo_state()
    } else {
        log::info!("Loading initial state from {}", args.state.display());
        let state_file = std::fs::File::open(&args.state)?;
        let reader = std::io::BufReader::new(state_file);
        serde_json::from_reader(reader)?
    };

    let mut file = std::fs::File::create("used_state.json")?;
    let writer = std::io::BufWriter::new(&mut file);
    serde_json::to_writer_pretty(writer, &initial_state)?;
    log::info!("Wrote used initial state to used_state.json");

    let mut integrator = Integrator::new(args.combats, roller, initial_state.clone());

    log::info!("Running {} combats...", args.combats);

    let state_tree = integrator.run()?;

    log::info!(
        "Simulation complete: {} combats run in {} seconds ({:.2} combats/sec)",
        integrator.combats_run(),
        integrator.elapsed_time().to_std().unwrap().as_secs_f64(),
        integrator.combats_per_second()
    );

    let total_combats = integrator.combats_run();
    let state_tree_stats = state_tree.compute_statistics();

    let stats = Statistics {
        initial_state,
        total_combats,
        state_tree,
        state_tree_stats,
    };

    let stats_file = std::fs::File::create(&args.output)?;
    let writer = std::io::BufWriter::new(stats_file);
    serde_json::to_writer(writer, &stats)?;
    log::info!("Statistics written to {}", args.output.display());

    Ok(())
}
