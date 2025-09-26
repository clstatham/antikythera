use rustc_hash::FxHashMap;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    stats: FxHashMap<Stat, u32>,
}

impl Default for Stats {
    fn default() -> Self {
        let mut stats = FxHashMap::default();
        for stat in Stat::all() {
            stats.insert(stat, 10);
        }
        Stats { stats }
    }
}

impl Stats {
    pub fn with_stat(mut self, stat: Stat, value: u32) -> Self {
        self.stats.insert(stat, value);
        self
    }

    pub fn get(&self, stat: Stat) -> u32 {
        self.stats.get(&stat).copied().unwrap_or(10)
    }

    pub fn get_mut(&mut self, stat: Stat) -> &mut u32 {
        self.stats.entry(stat).or_insert(10)
    }

    pub fn set(&mut self, stat: Stat, value: u32) {
        self.stats.insert(stat, value);
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
