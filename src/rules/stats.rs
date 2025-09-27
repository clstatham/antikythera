use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Stat {
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
}

impl Stat {
    pub fn all() -> Vec<Stat> {
        vec![
            Stat::Strength,
            Stat::Dexterity,
            Stat::Constitution,
            Stat::Intelligence,
            Stat::Wisdom,
            Stat::Charisma,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Stats {
    strength: u32,
    dexterity: u32,
    constitution: u32,
    intelligence: u32,
    wisdom: u32,
    charisma: u32,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            strength: 10,
            dexterity: 10,
            constitution: 10,
            intelligence: 10,
            wisdom: 10,
            charisma: 10,
        }
    }
}

impl Stats {
    pub fn with_stat(mut self, stat: Stat, value: u32) -> Self {
        self.set(stat, value);
        self
    }

    pub fn get(&self, stat: Stat) -> u32 {
        match stat {
            Stat::Strength => self.strength,
            Stat::Dexterity => self.dexterity,
            Stat::Constitution => self.constitution,
            Stat::Intelligence => self.intelligence,
            Stat::Wisdom => self.wisdom,
            Stat::Charisma => self.charisma,
        }
    }

    pub fn get_mut(&mut self, stat: Stat) -> &mut u32 {
        match stat {
            Stat::Strength => &mut self.strength,
            Stat::Dexterity => &mut self.dexterity,
            Stat::Constitution => &mut self.constitution,
            Stat::Intelligence => &mut self.intelligence,
            Stat::Wisdom => &mut self.wisdom,
            Stat::Charisma => &mut self.charisma,
        }
    }

    pub fn set(&mut self, stat: Stat, value: u32) {
        *self.get_mut(stat) = value;
    }

    pub fn modifier(&self, stat: Stat) -> i32 {
        self.get(stat) as i32 / 2 - 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stat_block_default() {
        let stats = Stats::default();
        for stat in Stat::all() {
            assert_eq!(stats.get(stat), 10);
            assert_eq!(stats.modifier(stat), 0);
        }
    }

    #[test]
    fn test_stat_block_with_stat() {
        let stats = Stats::default().with_stat(Stat::Strength, 16);
        assert_eq!(stats.get(Stat::Strength), 16);
        assert_eq!(stats.modifier(Stat::Strength), 3);
    }

    #[test]
    fn test_stat_block_set() {
        let mut stats = Stats::default();
        stats.set(Stat::Dexterity, 14);
        assert_eq!(stats.get(Stat::Dexterity), 14);
        assert_eq!(stats.modifier(Stat::Dexterity), 2);
    }

    #[test]
    fn test_stat_block_modifier() {
        let stats = Stats::default()
            .with_stat(Stat::Constitution, 8)
            .with_stat(Stat::Intelligence, 18);
        assert_eq!(stats.modifier(Stat::Constitution), -1);
        assert_eq!(stats.modifier(Stat::Intelligence), 4);
    }
}
