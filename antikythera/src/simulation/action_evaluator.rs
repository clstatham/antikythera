use crate::{
    rules::{
        actions::{Action, ActionTaken, AttackAction, UnarmedStrikeAction},
        actor::ActorId,
        items::{ItemId, ItemType},
    },
    simulation::{
        logging::{ExtraLogEntry, LogEntry},
        state::State,
        transition::Transition,
    },
    statistics::roller::Roller,
};

#[derive(Debug, Clone)]
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
        logs.push(LogEntry::Transition(Transition::ActionEconomyUsed {
            target: actor.id,
            action_type: action.action_type,
        }));

        logs.push(LogEntry::Extra(ExtraLogEntry::Action(action.clone())));

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
                logs.push(LogEntry::Extra(ExtraLogEntry::Roll(attack_result.clone())));

                let attack_hits = attack_result.meets_dc(target.armor_class as i32);
                let attack_crits = attack_result.is_critical_success();

                if attack_hits {
                    let damage_roll = if attack_crits {
                        actor.plan_unarmed_strike_crit_damage()
                    } else {
                        actor.plan_unarmed_strike_damage()
                    };
                    let damage_result = damage_roll.roll(rng)?;
                    logs.push(LogEntry::Extra(ExtraLogEntry::Roll(damage_result.clone())));

                    logs.push(LogEntry::Extra(ExtraLogEntry::AttackHit {
                        attacker: actor.id,
                        target: target.id,
                        weapon: ItemId(0), // Unarmed strike has no item ID
                    }));

                    // apply damage to target
                    logs.push(LogEntry::Transition(Transition::HealthModification {
                        target: target.id,
                        delta: -damage_result.total,
                    }));

                    if target.health <= damage_result.total {
                        logs.push(LogEntry::Extra(ExtraLogEntry::ActorDowned {
                            actor: target.id,
                        }));
                    }
                } else {
                    logs.push(LogEntry::Extra(ExtraLogEntry::AttackMiss {
                        attacker: actor.id,
                        target: target.id,
                        weapon: ItemId(0), // Unarmed strike has no item ID
                    }));
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
                logs.push(LogEntry::Extra(ExtraLogEntry::Roll(attack_result.clone())));

                let attack_hits = attack_result.meets_dc(target.armor_class as i32);

                if attack_hits {
                    logs.push(LogEntry::Extra(ExtraLogEntry::AttackHit {
                        attacker: actor.id,
                        target: target.id,
                        weapon: *weapon_used_id,
                    }));

                    let damage_roll = if attack_result.is_critical_success() {
                        weapon_used
                            .critical_damage
                            .as_ref()
                            .unwrap_or(&weapon_used.damage)
                    } else {
                        &weapon_used.damage
                    };

                    let damage_result = damage_roll.roll(rng)?;
                    logs.push(LogEntry::Extra(ExtraLogEntry::Roll(damage_result.clone())));

                    // apply damage to target
                    // todo: calculate resistances, vulnerabilities, temporary hit points, etc.
                    logs.push(LogEntry::Transition(Transition::HealthModification {
                        target: target.id,
                        delta: -damage_result.total,
                    }));

                    if target.health <= damage_result.total {
                        logs.push(LogEntry::Extra(ExtraLogEntry::ActorDowned {
                            actor: target.id,
                        }));
                    }
                } else {
                    logs.push(LogEntry::Extra(ExtraLogEntry::AttackMiss {
                        attacker: actor.id,
                        target: target.id,
                        weapon: *weapon_used_id,
                    }));
                }
            }
            action => todo!("Handle {:?} action", action),
        }

        Ok(logs)
    }
}
