use derive_more::{Deref, From, Into};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::rules::{
    dice::{RollFormula, RollPlan, RollSettings},
    spells::SpellId,
};

#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize, From, Into,
)]
pub struct ItemId(pub u32);

impl ItemId {
    pub fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &crate::simulation::state::SimulationState,
    ) -> std::fmt::Result {
        for actor in state.actors.values() {
            if let Some(entry) = actor.inventory.items.get(self) {
                return write!(f, "{} (ID: {})", entry.item.name, self.0);
            }
        }
        write!(f, "Unarmed Strike (ID: {})", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Potion(Potion),
    Scroll(Scroll),
    Weapon(Weapon),
    Armor(Armor),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub item_type: ItemType,
}

impl Item {
    #[cfg(test)]
    pub fn test_sword() -> Self {
        Self {
            id: ItemId(1),
            name: "Test Sword".to_string(),
            item_type: ItemType::Weapon(Weapon::test_sword()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Potion {
    pub healing_amount: RollPlan,
}

impl Potion {
    #[cfg(test)]
    pub fn test_potion() -> Self {
        Self {
            healing_amount: RollPlan {
                num_dice: 2,
                die_size: 4,
                modifier: 2,
                settings: RollSettings::default(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scroll {
    pub spell_id: SpellId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub attack_bonus: i32,
    pub damage: RollFormula,
    pub critical_damage: Option<RollFormula>,
    pub range: Option<u32>, // in feet, None for melee
}

impl Weapon {
    pub fn is_melee(&self) -> bool {
        self.range.is_none()
    }

    pub fn is_ranged(&self) -> bool {
        self.range.is_some()
    }

    pub fn plan_attack_roll(&self, settings: RollSettings) -> RollPlan {
        RollPlan {
            num_dice: 1,
            die_size: 20,
            modifier: self.attack_bonus,
            settings,
        }
    }

    #[cfg(test)]
    pub fn test_sword() -> Self {
        Self {
            attack_bonus: 5,
            damage: RollFormula {
                rolls: vec![RollPlan {
                    num_dice: 1,
                    die_size: 8,
                    modifier: 3,
                    settings: RollSettings::default(),
                }],
                flat_modifier: 0,
            },
            critical_damage: None,
            range: None,
        }
    }
}

pub struct WeaponBuilder {
    weapon: Weapon,
}

impl WeaponBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            weapon: Weapon {
                attack_bonus: 0,
                damage: RollFormula {
                    rolls: vec![],
                    flat_modifier: 0,
                },
                critical_damage: None,
                range: None,
            },
        }
    }

    pub fn attack_bonus(mut self, bonus: i32) -> Self {
        self.weapon.attack_bonus = bonus;
        self
    }

    pub fn damage(mut self, damage: impl Into<RollFormula>) -> Self {
        self.weapon.damage = damage.into();
        self
    }

    pub fn critical_damage(mut self, critical_damage: impl Into<RollFormula>) -> Self {
        self.weapon.critical_damage = Some(critical_damage.into());
        self
    }

    pub fn range(mut self, range: u32) -> Self {
        self.weapon.range = Some(range);
        self
    }

    pub fn build(self) -> Weapon {
        self.weapon
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Armor {
    pub ac_bonus: u32,
    pub stealth_disadvantage: bool,
}

impl Armor {
    #[cfg(test)]
    pub fn test_armor() -> Self {
        Self {
            ac_bonus: 2,
            stealth_disadvantage: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryEntry {
    pub item: Item,
    pub quantity: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EquipSlot {
    Head,
    Chest,
    Legs,
    Feet,
    Hands,
    Shield,
    Accessory,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquippedItems {
    pub slots: FxHashMap<EquipSlot, ItemId>,
}

impl EquippedItems {
    pub fn equip(&mut self, slot: EquipSlot, item_id: ItemId) {
        self.slots.insert(slot, item_id);
    }

    pub fn unequip(&mut self, slot: EquipSlot) -> Option<ItemId> {
        self.slots.remove(&slot)
    }

    pub fn get_equipped(&self, slot: EquipSlot) -> Option<ItemId> {
        self.slots.get(&slot).cloned()
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Deref)]
pub struct Inventory {
    pub items: FxHashMap<ItemId, InventoryEntry>,
}

impl Inventory {
    pub fn add_item(&mut self, item: Item, quantity: u32) {
        let entry = self
            .items
            .entry(item.id)
            .or_insert(InventoryEntry { item, quantity: 0 });
        entry.quantity += quantity;
    }

    pub fn remove_item(&mut self, item_id: ItemId, quantity: u32) -> Option<Item> {
        let mut remove = false;
        if let Some(entry) = self.items.get_mut(&item_id)
            && entry.quantity >= quantity
        {
            entry.quantity -= quantity;
            if entry.quantity == 0 {
                remove = true;
            }
        }

        if remove {
            self.items.remove(&item_id).map(|entry| entry.item)
        } else {
            self.items.get(&item_id).map(|entry| entry.item.clone())
        }
    }

    pub fn has_item(&self, item_id: ItemId, quantity: u32) -> bool {
        self.items
            .get(&item_id)
            .is_some_and(|entry| entry.quantity >= quantity)
    }
}
