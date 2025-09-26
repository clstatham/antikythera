# Antikythera - D&D Combat Simulator

## Architecture Overview

Antikythera is a scriptable D&D 5e combat simulation engine built in Rust with a modular architecture:

- **`rules/`**: Core D&D mechanics (dice rolling, actors, actions, spells, stats)
- **`simulation/`**: Combat state management and execution engine  
- **`statistics/`**: Mathematical utilities for probability modeling

There are also auxiliary modules, e.g. for parsing dice notation from strings.

### Core Data Flow

1. **Roll Planning**: Actions create `RollPlan` structs that specify dice, modifiers, and settings
2. **Simulation State**: `SimulationState` manages actors, initiative order, and turn progression
3. **Turn Execution**: `Turn` contains `ActionTaken` sequences applied via `SimulationState::apply_turn()`
4. **Logging**: All events recorded as `LogEntry` enum variants for replay/analysis

## Key Patterns

### Dice Rolling System
- All randomness goes through `Roller` (seeded RNG for reproducibility)
- `RollPlan` separates roll specification from execution
- `RollSettings` handle advantage/disadvantage, min/max values, rerolls
- Critical hits/failures only apply to d20s (see `dice.rs:78-84`)

```rust
// Typical roll pattern
let plan = actor.plan_skill_check(Skill::Athletics, RollSettings::default());
let result = plan.roll(&mut rng)?;
```

### Actor System
- Actors have `ActorId` handles, stored in `BTreeMap` for stable iteration
- Stats use D&D ability score system with modifier calculation
- Action economy tracked per-turn (action/bonus action/reaction/movement)

### Action Resolution
- Actions are `enum Action` variants (Attack, CastSpell, etc.)
- `ActionTaken` wraps actions with their economy type
- State changes applied via `SimulationState::apply_action()`

## Development Guidelines

### Adding New Rules
- Follow the `rules/` module structure - new mechanics go in dedicated files
- Use `derive_more::{From, Into}` for ID wrapper types (`ActorId`, `SpellId`, etc.)
- All rule components should be `Serialize + Deserialize` for save/load

### Completeness
- Do not generate placeholder code or comments; Generate what the code should look like after all features are implemented (I will handle stubs and TODOs myself)

### Testing Patterns
- Use `Roller::test_rng()` for deterministic tests
- Test files include integration tests with full combat scenarios
- Property-based tests validate statistical properties (see `dice.rs` tests)

### Statistical Components
- `pmf.rs` provides probability mass function utilities
- `hit_model.rs` models attack success rates  
- Always validate edge cases (0 dice, invalid modifiers)

## Build & Test

```bash
cargo test                    # Run all tests
cargo test dice               # Test specific module
cargo check                   # Fast compile check
```

## Common Gotchas

- Initiative order uses `BTreeMap` keys, not insertion order
- Critical hits only trigger on natural 20s on d20s, regardless of modifiers  
- `reroll_dice_below` changes the distribution, not just the minimum
- Action economy resets per round, not per turn
- Actor health can go negative (massive damage rules)

## Extension Points

- New actions: Add variants to `Action` enum + handling in `apply_action()`
- AI policies: Implement `Policy` trait for automated decision making
- Custom dice mechanics: Extend `RollSettings` with new options
- Spell effects: Use `SpellTarget` enum for area/single target spells