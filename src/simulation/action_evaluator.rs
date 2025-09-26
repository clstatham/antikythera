use crate::{
    rules::{
        actions::{Action, ActionTaken, AttackAction, UnarmedStrikeAction},
        actor::ActorId,
        items::{ItemId, ItemType},
    },
    simulation::{logging::LogEntry, state::State, transition::Transition},
    statistics::roller::Roller,
};

pub struct ActionEvaluator;

impl ActionEvaluator {
    pub fn evaluate_action(
        &self,
        actor: ActorId,
        action: &ActionTaken,
        state: &State,
        rng: &mut Roller,
    ) -> anyhow::Result<Vec<LogEntry>> {
        let mut logs = Vec::new();

        let Some(actor) = state.get_actor(actor) else {
            anyhow::bail!("Actor not found in simulation state");
        };

        if !actor.action_economy.can_take_action(action.action_type) {
            return Ok(logs);
        }

        // the action was successfully taken at this point
        logs.push(LogEntry::Transition(Transition::ActionUsed {
            target: actor.id,
            action_type: action.action_type,
        }));

        logs.push(LogEntry::Action(action.clone()));

        match &action.action {
            Action::Wait => {}
            Action::UnarmedStrike(UnarmedStrikeAction {
                target,
                attack_roll_settings,
            }) => {
                let target = state
                    .actors
                    .get(target)
                    .ok_or_else(|| anyhow::anyhow!("Target actor not found"))?;

                let attack_roll = actor.plan_unarmed_strike_roll(*attack_roll_settings);
                let attack_result = attack_roll.roll(rng)?;
                logs.push(LogEntry::Roll(attack_result.clone()));

                let attack_hits = attack_result.meets_dc(target.armor_class as i32);
                let attack_crits = attack_result.is_critical_success();

                if attack_hits {
                    let damage_roll = if attack_crits {
                        actor.plan_unarmed_strike_crit_damage()
                    } else {
                        actor.plan_unarmed_strike_damage()
                    };
                    let damage_result = damage_roll.roll(rng)?;
                    logs.push(LogEntry::Roll(damage_result.clone()));

                    logs.push(LogEntry::AttackHit {
                        attacker: actor.id,
                        target: target.id,
                        weapon: ItemId(0), // Unarmed strike has no item ID
                    });

                    // apply damage to target
                    logs.push(LogEntry::Transition(Transition::HealthModification {
                        target: target.id,
                        delta: -damage_result.total,
                    }));

                    if target.health <= damage_result.total {
                        logs.push(LogEntry::ActorDowned { actor: target.id });
                    }
                } else {
                    logs.push(LogEntry::AttackMiss {
                        attacker: actor.id,
                        target: target.id,
                        weapon: ItemId(0), // Unarmed strike has no item ID
                    });
                }
            }
            Action::Attack(AttackAction {
                weapon_used: weapon_used_id,
                target,
                attack_roll_settings,
            }) => {
                let target = state
                    .actors
                    .get(target)
                    .ok_or_else(|| anyhow::anyhow!("Target actor not found"))?;

                let weapon_used = &actor
                    .inventory
                    .get(weapon_used_id)
                    .ok_or_else(|| {
                        anyhow::anyhow!("Actor does not have the weapon used for the attack")
                    })?
                    .item;

                let ItemType::Weapon(weapon_used) = &weapon_used.item_type else {
                    return Err(anyhow::anyhow!("Item used for attack is not a weapon"));
                };

                let attack_roll = actor.plan_attack_roll(weapon_used, *attack_roll_settings)?;
                let attack_result = attack_roll.roll(rng)?;
                logs.push(LogEntry::Roll(attack_result.clone()));

                let attack_hits = attack_result.meets_dc(target.armor_class as i32);

                if attack_hits {
                    logs.push(LogEntry::AttackHit {
                        attacker: actor.id,
                        target: target.id,
                        weapon: *weapon_used_id,
                    });

                    let damage_roll = if attack_result.is_critical_success() {
                        weapon_used
                            .critical_damage
                            .as_ref()
                            .unwrap_or(&weapon_used.damage)
                    } else {
                        &weapon_used.damage
                    };

                    let damage_result = damage_roll.roll(rng)?;
                    logs.push(LogEntry::Roll(damage_result.clone()));

                    // apply damage to target
                    // todo: calculate resistances, vulnerabilities, temporary hit points, etc.
                    logs.push(LogEntry::Transition(Transition::HealthModification {
                        target: target.id,
                        delta: -damage_result.total,
                    }));

                    if target.health <= damage_result.total {
                        logs.push(LogEntry::ActorDowned { actor: target.id });
                    }
                } else {
                    logs.push(LogEntry::AttackMiss {
                        attacker: actor.id,
                        target: target.id,
                        weapon: *weapon_used_id,
                    });
                }
            }
            action => todo!("Handle {:?} action", action),
        }

        Ok(logs)
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::{
        actions::ActionEconomyUsage,
        actor::Actor,
        dice::{Advantage, RollSettings},
        items::Item,
    };

    use super::*;

    #[test]
    fn test_evaluate_attack_action() {
        // Setup a simple simulation state with two actors and a weapon
        let mut state = State::default();

        let actor1 = Actor::test_actor(1, "Attacker");
        let actor1_id = state.add_actor(actor1);

        let actor2 = Actor::test_actor(2, "Defender");
        let actor2_id = state.add_actor(actor2);

        // Create a weapon and add it to the attacker's inventory
        let weapon = Item::test_sword();
        state
            .actors
            .get_mut(&actor1_id)
            .unwrap()
            .inventory
            .add_item(weapon.clone(), 1);

        // Create an attack action
        let attack_action = ActionTaken {
            actor: actor1_id,
            action_type: ActionEconomyUsage::Action,
            action: Action::Attack(AttackAction {
                weapon_used: weapon.id,
                target: actor2_id,
                attack_roll_settings: RollSettings {
                    advantage: Advantage::Advantage,
                    minimum_die_value: None,
                    maximum_die_value: None,
                    reroll_dice_below: None,
                },
            }),
        };

        let evaluator = ActionEvaluator;
        let mut rng = Roller::test_rng();
        let logs = evaluator
            .evaluate_action(actor1_id, &attack_action, &state, &mut rng)
            .unwrap();
        assert!(!logs.is_empty());

        // Check that the logs contain expected entries
        let roll_logs: Vec<_> = logs
            .iter()
            .filter(|log| matches!(log, LogEntry::Roll(_)))
            .collect();
        assert!(!roll_logs.is_empty());
        let transition_logs: Vec<_> = logs
            .iter()
            .filter(|log| matches!(log, LogEntry::Transition(_)))
            .collect();
        assert!(!transition_logs.is_empty());

        let json = serde_json::to_string_pretty(&logs).unwrap();
        println!("{}", json);
    }
}
