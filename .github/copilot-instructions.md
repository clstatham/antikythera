# Antikythera Workspace Guide

## Workspace Layout
- `antikythera/` – core simulation engine library crate; `lib.rs` exposes modules and the `prelude` re-export for downstream consumers.
- `antikythera-cli/` – command-line harness for running large batches of combats and exporting `Statistics` snapshots.
- `antikythera-gui/` – `eframe`/`egui` desktop client with tabs for state editing, simulation, and statistical analysis.
- Shared configuration lives at the workspace root (`Cargo.toml`, `.github/`, etc.).

## Core Engine (`antikythera` crate)
- `rules/` – D&D 5e rules primitives: actors, stats, skills, saves, items, dice, actions, spells, damage, death, and derived logic such as proficiencies.
- `simulation/` – runtime systems (`State`, `Executor`, `Policy`, `ActionEvaluator`, `Transition`, logging) that advance combat rounds.
- `statistics/` – tooling for repeated simulations (`Roller`, `Integrator`, `StateTree`, `Query`, probability summaries).
- `roll_parser.rs` – converts strings like `"2d6+3"` into `RollPlan` instances; use it instead of hand-parsing dice.
- `utils.rs` – shared helpers such as `ProtectedCell<T>`, which wraps mutable state and forces explicit mutation via `ProtectedCell::get_mut`.

### Simulation Flow
1. Build a `State` with actors and items (typically via `ActorBuilder`, `WeaponBuilder`, and `State::add_*`).
2. `Executor::new` seeds a `ProtectedCell<State>`, `SimulationLog`, `ActionEvaluator`, and a `Policy` (default `RandomPolicy`).
3. `Executor::begin_combat` records initiative rolls (`Transition::InitiativeRoll`) and applies them with `Transition::apply`.
4. Each turn, `Policy::take_action` returns an `ActionTaken` for every economy slot (`Action`, `BonusAction`), using `Roller` for randomness.
5. `ActionEvaluator::evaluate_action` resolves the action, emitting `LogEntry::Transition` for state changes and `LogEntry::Extra` for narrative/roll data.
6. The CLI/GUI `Integrator` replays those logs into a `StateTree`, deduplicating states, counting hits, and computing probabilities.

### Dice & Randomness
- All randomness flows through `rules::dice::{RollPlan, RollSettings, RollResult}` and `statistics::roller::Roller`.
- Advantage/disadvantage, rerolls, and min/max clamps operate on each individual die result; critical hits only trigger on natural 20s from d20 rolls.
- Use `Roller::fork()` when spawning new threads or child RNGs, `Roller::from_seed` for reproducible runs, and `Roller::test_rng()` in unit tests.

### Actors, Inventory, Action Economy
- Construct actors with `ActorBuilder`; IDs are `ActorId` wrappers stored in the `State`'s `BTreeMap` for deterministic iteration.
- Items live in `State::items`; create them (e.g., via `WeaponBuilder`) before assigning them to an actor's inventory.
- `ActionEconomy::can_take_action` guards duplicate usage; when the action resolves, emit `Transition::ActionEconomyUsed` so logs and the state tree stay consistent.
- Actors belong to integer `group`s; `State::enemies_of`/`allies_of` drive AI and win-condition checks.

### Logging & Transitions
- `simulation::logging::LogEntry` wraps either a `Transition` (authoritative state change) or `ExtraLogEntry` (diagnostics for UI/debugging).
- Always mutate the simulation state through `Transition::apply`; bypassing it desynchronizes the executor, logs, and `StateTree`.
- `Transition::is_quiet`/`emoji` help presentation layers decide what to display; add variants when introducing new transition types.

## Statistics & Query System
- `statistics::integration::Integrator` runs repeated combats, tracks elapsed time, and exposes progress counters for the GUI.
- `statistics::state_tree::StateTree` deduplicates states, tracks how often nodes/edges occur, and derives `StateTreeStats` (probabilities, branching factor, max depth).
- Implement the `statistics::query::Query` trait to add new analytics; existing helpers include `OutcomeConditionProbability` (Rust closures) and the Lua-backed `ScriptProbabilityQuery` used by the GUI.

## Client Crates
### CLI (`antikythera-cli`)
- `src/main.rs` uses `clap` to parse arguments, load or generate a `State`, run an `Integrator`, and serialize `Statistics` to JSON.
- `demo_state()` showcases a minimal encounter; keep it in sync with engine mechanics when rules change.

### GUI (`antikythera-gui`)
- Built on `eframe`; `app::App` switches between Home, `state_editor`, `simulation`, and `analysis` modes while sharing `State`/`Statistics` instances.
- The state editor tab creates, clones, edits, and saves serialized `State`s (prompting on unsaved changes).
- The simulation tab spawns an `Integrator` on a background thread, reports progress, and persists results with `serde_json`.
- The analysis tab loads saved `Statistics`, executes Lua queries via `mlua`, and appends formatted metrics to a scrolling table.

## Development Guidelines
- Keep mechanics in their owning crate: data/modeling in `rules`, runtime orchestration in `simulation`, post-processing in `statistics`.
- Derive `Serialize`/`Deserialize` on new stateful types, wire them through builders, and update GUI/CLI serializers.
- Mutate shared state explicitly with `ProtectedCell::get_mut` and `Transition::apply` so logs and probability calculations remain accurate.
- Extend the `Policy` trait or `ActionEvaluator` when adding new actions; ensure every branch produces deterministic transitions and meaningful log entries.
- Do not emit placeholder functions or TODO comments—generate the final implementation.
- Prefer deterministic tests using `Roller::test_rng()`; integration-style tests can exercise `Executor` or `Integrator` on representative encounters.

## Build & Test
```bash
cargo check --workspace
cargo test -p antikythera
cargo run -p antikythera-cli -- --help
cargo run -p antikythera-gui
```

## Common Gotchas
- `State::is_combat_over` stops when only one living `group` remains; assign group IDs consistently (0 = player party, 1 = opponents, etc.).
- Initiative is recomputed whenever `Transition::InitiativeRoll` fires; set `Actor.initiative` before logging if you need deterministic ordering.
- Action economy resets at `Transition::BeginTurn`; remember to consume it via transitions when resolving custom actions.
- Critical damage should come from the weapon's `critical_damage` plan; fall back to normal `damage` when not provided.
- `StateTree` hit counters saturate; only interact with states through `StateTree::add_node`/`add_edge` to keep counts correct.
