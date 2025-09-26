use derive_more::{Deref, From, Into};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::rules::{dice::RollPlan, skills::SkillProficiency, spells::SpellId};

#[derive(
    Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash, Serialize, Deserialize, From, Into,
)]
pub struct ItemId(pub u32);

impl ItemId {
    pub fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &crate::simulation::state::State,
    ) -> std::fmt::Result {
        for actor in state.actors.values() {
            if let Some(entry) = actor.inventory.items.get(self) {
                return write!(f, "{}", entry.item.name);
            }
        }
        write!(f, "Unarmed Strike")
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
        use crate::rules::dice::RollSettings;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponType {
    Club,
    Dagger,
    Greatclub,
    Handaxe,
    Javelin,
    LightHammer,
    Mace,
    Quarterstaff,
    Sickle,
    Spear,
    CrossbowLight,
    Dart,
    Shortbow,
    Sling,
    Battleaxe,
    Flail,
    Glaive,
    Greataxe,
    Greatsword,
    Halberd,
    Lance,
    Longsword,
    Maul,
    Morningstar,
    Pike,
    Rapier,
    Scimitar,
    Shortsword,
    Trident,
    WarPick,
    Warhammer,
    Whip,
    Blowgun,
    CrossbowHeavy,
    Longbow,
    Net,
}

impl WeaponType {
    pub fn all() -> &'static [WeaponType] {
        use WeaponType::*;
        &[
            Club,
            Dagger,
            Greatclub,
            Handaxe,
            Javelin,
            LightHammer,
            Mace,
            Quarterstaff,
            Sickle,
            Spear,
            CrossbowLight,
            Dart,
            Shortbow,
            Sling,
            Battleaxe,
            Flail,
            Glaive,
            Greataxe,
            Greatsword,
            Halberd,
            Lance,
            Longsword,
            Maul,
            Morningstar,
            Pike,
            Rapier,
            Scimitar,
            Shortsword,
            Trident,
            WarPick,
            Warhammer,
            Whip,
            Blowgun,
            CrossbowHeavy,
            Longbow,
            Net,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Weapon {
    pub weapon_type: WeaponType,
    pub attack_bonus: i32,
    pub damage: RollPlan,
    pub critical_damage: Option<RollPlan>,
    pub range: Option<u32>, // in feet, None for melee
}

impl Weapon {
    pub fn is_melee(&self) -> bool {
        self.range.is_none()
    }

    pub fn is_ranged(&self) -> bool {
        self.range.is_some()
    }

    #[cfg(test)]
    pub fn test_sword() -> Self {
        use crate::rules::dice::RollSettings;
        Self {
            attack_bonus: 1,
            weapon_type: WeaponType::Longsword,
            damage: RollPlan {
                num_dice: 1,
                die_size: 8,
                modifier: 3,
                settings: RollSettings::default(),
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
    pub fn new(weapon_type: WeaponType) -> Self {
        Self {
            weapon: Weapon {
                attack_bonus: 0,
                weapon_type,
                damage: RollPlan {
                    num_dice: 0,
                    die_size: 0,
                    modifier: 0,
                    settings: Default::default(),
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

    pub fn damage(mut self, damage: impl Into<RollPlan>) -> Self {
        self.weapon.damage = damage.into();
        self
    }

    pub fn critical_damage(mut self, critical_damage: impl Into<RollPlan>) -> Self {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeaponProficiency {
    None,
    HalfProficient,
    Proficient,
}

impl From<WeaponProficiency> for SkillProficiency {
    fn from(prof: WeaponProficiency) -> Self {
        match prof {
            WeaponProficiency::None => SkillProficiency::None,
            WeaponProficiency::HalfProficient => SkillProficiency::HalfProficient,
            WeaponProficiency::Proficient => SkillProficiency::Proficient,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeaponProficiencies {
    proficiencies: FxHashMap<WeaponType, WeaponProficiency>,
}

impl Default for WeaponProficiencies {
    fn default() -> Self {
        let mut proficiencies = FxHashMap::default();
        for weapon_type in WeaponType::all() {
            proficiencies.insert(*weapon_type, WeaponProficiency::None);
        }
        WeaponProficiencies { proficiencies }
    }
}

impl WeaponProficiencies {
    pub fn with_proficiency(
        mut self,
        weapon_type: WeaponType,
        proficiency: WeaponProficiency,
    ) -> Self {
        self.proficiencies.insert(weapon_type, proficiency);
        self
    }

    pub fn get(&self, weapon_type: WeaponType) -> WeaponProficiency {
        *self
            .proficiencies
            .get(&weapon_type)
            .unwrap_or(&WeaponProficiency::None)
    }

    pub fn set(&mut self, weapon_type: WeaponType, proficiency: WeaponProficiency) {
        self.proficiencies.insert(weapon_type, proficiency);
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
