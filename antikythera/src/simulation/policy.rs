use crate::{
    prelude::ActionType,
    rules::{
        actions::{Action, ActionEconomyUsage, ActionTaken, AttackAction, UnarmedStrikeAction},
        actor::ActorId,
        items::ItemInner,
    },
    simulation::{roller::Roller, state::State},
};

use rand::Rng;
use rand::distr::weighted::WeightedIndex;
use rand_distr::Distribution;
use serde::{Deserialize, Serialize};

pub struct WeightedProbability<T> {
    pub items: Vec<(T, i32)>,
    distr: WeightedIndex<i32>,
}

impl<T: Clone> WeightedProbability<T> {
    pub fn new(items: Vec<(T, i32)>) -> Self {
        let distr = WeightedIndex::new(items.iter().map(|&(_, weight)| weight)).unwrap();
        Self { items, distr }
    }

    pub fn sample(&self, rng: &mut impl Rng) -> &T {
        let index = self.distr.sample(rng);
        &self.items[index].0
    }
}

#[derive(Debug, Clone, Default)]
pub struct PolicyBuilder {
    policy: Policy,
}

impl PolicyBuilder {
    pub fn new() -> Self {
        Self {
            policy: Policy::default(),
        }
    }

    pub fn action_weight(mut self, action: ActionType, weight: i32) -> Self {
        if let Some((_, existing_weight)) = self
            .policy
            .action_weights
            .iter_mut()
            .find(|(a, _)| *a == action)
        {
            *existing_weight = weight;
        } else {
            self.policy.action_weights.push((action, weight));
        }
        self
    }

    pub fn target_weight(mut self, actor_id: ActorId, weight: i32) -> Self {
        if let Some((_, existing_weight)) = self
            .policy
            .target_weights
            .iter_mut()
            .find(|(id, _)| *id == actor_id)
        {
            *existing_weight = weight;
        } else {
            self.policy.target_weights.push((actor_id, weight));
        }
        self
    }

    pub fn build(self) -> Policy {
        self.policy
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Policy {
    pub action_weights: Vec<(ActionType, i32)>,
    pub target_weights: Vec<(ActorId, i32)>,
}

impl Policy {
    pub fn take_action(
        &self,
        action_economy_usage: ActionEconomyUsage,
        actor: ActorId,
        state: &State,
        rng: &mut Roller,
    ) -> anyhow::Result<ActionTaken> {
        if action_economy_usage != ActionEconomyUsage::Action {
            return Ok(ActionTaken {
                actor,
                action: Action::Wait,
                action_economy_usage,
            });
        }

        let enemies = state.possible_targets(actor);
        if enemies.is_empty() {
            return Ok(ActionTaken {
                actor,
                action: Action::Wait,
                action_economy_usage,
            });
        }

        let mut target_weights = vec![];
        for enemy in enemies {
            let weight = self
                .target_weights
                .iter()
                .find(|(id, _)| *id == enemy)
                .map(|(_, weight)| *weight)
                .unwrap_or(1);
            target_weights.push((enemy, weight));
        }
        let target_table = WeightedProbability::new(target_weights);
        let target = *target_table.sample(rng.rng());

        let actor = state.get_actor(actor).unwrap();

        let mut weapon_used = None;
        for item_id in actor.inventory.items.keys() {
            if let Some(item) = state.items.get(item_id)
                && let ItemInner::Weapon(_) = &item.inner
            {
                weapon_used = Some(*item_id);
                break;
            }
        }

        let mut action_weights = self.action_weights.clone();
        let possible_actions = state.possible_actions(actor.id);
        action_weights.retain(|(action_type_candidate, _)| match action_type_candidate {
            ActionType::Attack => weapon_used.is_some(),
            ActionType::UnarmedStrike => true,
            _ => false,
        });
        action_weights
            .retain(|(action_type_candidate, _)| possible_actions.contains(action_type_candidate));
        if action_weights.is_empty() {
            return Ok(ActionTaken {
                actor: actor.id,
                action: Action::Wait,
                action_economy_usage,
            });
        }
        let action_table = WeightedProbability::new(action_weights);
        let action_type = action_table.sample(rng.rng());

        let action = match action_type {
            ActionType::Wait => Action::Wait,
            ActionType::Attack => Action::Attack(AttackAction {
                weapon_used: weapon_used.unwrap(),
                target,
                attack_roll_settings: Default::default(),
            }),
            ActionType::UnarmedStrike => Action::UnarmedStrike(UnarmedStrikeAction {
                target,
                attack_roll_settings: Default::default(),
            }),
            _ => Action::Wait, // placeholder for other actions
        };

        Ok(ActionTaken {
            actor: actor.id,
            action,
            action_economy_usage,
        })
    }
}
