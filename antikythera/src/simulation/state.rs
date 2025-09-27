use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::rules::{
    actor::{Actor, ActorId},
    items::{Item, ItemId, ItemInner},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct State {
    pub turn: u64,
    pub actors: BTreeMap<ActorId, Actor>,
    pub next_actor_id: u32,
    pub items: BTreeMap<ItemId, Item>,
    pub next_item_id: u32,
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

    pub fn add_item(&mut self, name: &str, item: ItemInner) -> ItemId {
        let item_id = ItemId(self.next_item_id);
        self.next_item_id += 1;
        let item = Item {
            id: item_id,
            name: name.to_string(),
            inner: item,
        };
        self.items.insert(item_id, item);
        item_id
    }

    pub fn get_actor(&self, actor_id: ActorId) -> Option<&Actor> {
        self.actors.get(&actor_id)
    }

    pub fn get_actor_mut(&mut self, actor_id: ActorId) -> Option<&mut Actor> {
        self.actors.get_mut(&actor_id)
    }

    pub fn allies_of(&self, actor_id: ActorId) -> Option<Vec<ActorId>> {
        let actor = self.actors.get(&actor_id)?;
        let group_id = actor.group;
        let allies: Vec<ActorId> = self
            .actors
            .values()
            .filter(|a| a.group == group_id && a.id != actor_id)
            .map(|a| a.id)
            .collect();
        Some(allies)
    }

    pub fn enemies_of(&self, actor_id: ActorId) -> Vec<ActorId> {
        let mut enemies = BTreeSet::from_iter(self.actors.keys().cloned().collect::<Vec<_>>());
        if let Some(allies) = self.allies_of(actor_id) {
            for ally in allies {
                enemies.remove(&ally);
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
        let mut seen_groups = BTreeSet::new();
        for actor in self.actors.values() {
            if !seen_groups.contains(&actor.group) {
                seen_groups.insert(actor.group);
                remaining_groups += 1;
            }
        }

        remaining_groups <= 1
    }
}
