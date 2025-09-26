use rand::Rng;

use crate::{
    rules::{
        actions::{Action, ActionEconomyUsage, ActionTaken, AttackAction, UnarmedStrikeAction},
        actor::ActorId,
        items::ItemType,
    },
    simulation::state::SimulationState,
    statistics::roller::Roller,
};

pub trait Policy: 'static {
    fn take_action(
        &self,
        action_type: ActionEconomyUsage,
        actor: ActorId,
        state: &SimulationState,
        rng: &mut Roller,
    ) -> anyhow::Result<ActionTaken>;
}

#[derive(Debug, Clone)]
pub struct NoOpPolicy;

impl Policy for NoOpPolicy {
    fn take_action(
        &self,
        action_type: ActionEconomyUsage,
        actor: ActorId,
        _state: &SimulationState,
        _rng: &mut Roller,
    ) -> anyhow::Result<ActionTaken> {
        Ok(ActionTaken {
            actor,
            action: Action::Wait,
            action_type,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RandomPolicy;

impl RandomPolicy {
    fn random_action(&self, actor: ActorId, state: &SimulationState, rng: &mut Roller) -> Action {
        let enemies = state.enemies_of(actor);
        if enemies.is_empty() {
            return Action::Wait;
        }
        let target = enemies[rng.rng().random_range(0..enemies.len())];

        let actor = state.get_actor(actor).unwrap();

        let mut weapon_used = None;
        for (item_id, entry) in actor.inventory.iter() {
            if let ItemType::Weapon(_) = entry.item.item_type {
                weapon_used = Some(*item_id);
                break;
            }
        }

        if let Some(weapon_used) = weapon_used {
            Action::Attack(AttackAction {
                weapon_used,
                target,
                attack_roll_settings: Default::default(),
            })
        } else {
            Action::UnarmedStrike(UnarmedStrikeAction {
                target,
                attack_roll_settings: Default::default(),
            })
        }
    }
}

impl Policy for RandomPolicy {
    fn take_action(
        &self,
        action_type: ActionEconomyUsage,
        actor: ActorId,
        state: &SimulationState,
        rng: &mut Roller,
    ) -> anyhow::Result<ActionTaken> {
        if action_type != ActionEconomyUsage::Action {
            return Ok(ActionTaken {
                actor,
                action: Action::Wait,
                action_type,
            });
        }
        let action = self.random_action(actor, state, rng);

        Ok(ActionTaken {
            actor,
            action,
            action_type,
        })
    }
}
