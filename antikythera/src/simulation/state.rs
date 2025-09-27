use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::rules::{
    actor::{Actor, ActorId},
    items::{Item, ItemId, ItemType},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct State {
    pub turn: u64,
    pub actors: BTreeMap<ActorId, Actor>,
    pub next_actor_id: u32,
    pub items: BTreeMap<ItemId, Item>,
    pub next_item_id: u32,
    pub allied_groups: BTreeSet<Vec<ActorId>>,
    pub initiative_order: Vec<ActorId>,
    pub current_turn_index: Option<usize>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            turn: 0,
            actors: BTreeMap::new(),
            next_actor_id: 1,
            items: BTreeMap::new(),
            next_item_id: 1,
            allied_groups: BTreeSet::new(),
            initiative_order: Vec::new(),
            current_turn_index: None,
        }
    }

    pub fn add_actor(&mut self, mut actor: Actor) -> ActorId {
        let actor_id = ActorId(self.next_actor_id);
        self.next_actor_id += 1;
        actor.id = actor_id;
        self.actors.insert(actor_id, actor);
        actor_id
    }

    pub fn add_item(&mut self, name: &str, item: ItemType) -> Item {
        let item_id = ItemId(self.next_item_id);
        self.next_item_id += 1;
        let item = Item {
            id: item_id,
            name: name.to_string(),
            item_type: item,
        };
        self.items.insert(item_id, item.clone());
        item
    }

    pub fn get_actor(&self, actor_id: ActorId) -> Option<&Actor> {
        self.actors.get(&actor_id)
    }

    pub fn get_actor_mut(&mut self, actor_id: ActorId) -> Option<&mut Actor> {
        self.actors.get_mut(&actor_id)
    }

    pub fn add_ally_group(&mut self, group: Vec<ActorId>) {
        self.allied_groups.insert(group);
    }

    pub fn allies_of(&self, actor_id: ActorId) -> Option<&[ActorId]> {
        self.allied_groups
            .iter()
            .find(|&group| group.contains(&actor_id))
            .map(|v| v.as_slice())
    }

    pub fn enemies_of(&self, actor_id: ActorId) -> Vec<ActorId> {
        let mut enemies = BTreeSet::from_iter(self.actors.keys().cloned().collect::<Vec<_>>());
        if let Some(allies) = self.allies_of(actor_id) {
            for ally in allies {
                enemies.remove(ally);
            }
        }
        enemies.remove(&actor_id);
        enemies.into_iter().collect()
    }

    pub fn are_allies(&self, actor1: ActorId, actor2: ActorId) -> bool {
        if let Some(group) = self.allies_of(actor1) {
            group.contains(&actor2)
        } else {
            false
        }
    }

    pub fn are_enemies(&self, actor1: ActorId, actor2: ActorId) -> bool {
        !self.are_allies(actor1, actor2)
    }
    pub fn is_combat_over(&self) -> bool {
        // combat is over when only one allied group remains
        let mut remaining_groups = 0;
        for group in &self.allied_groups {
            if group.iter().any(|actor_id| {
                self.actors
                    .get(actor_id)
                    .is_some_and(|actor| actor.is_alive())
            }) {
                remaining_groups += 1;
            }
        }

        remaining_groups <= 1
    }
}
