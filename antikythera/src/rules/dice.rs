use serde::{Deserialize, Serialize};

use crate::statistics::roller::Roller;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Advantage {
    #[default]
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct RollSettings {
    pub advantage: Advantage,
    pub minimum_die_value: Option<u32>,
    pub maximum_die_value: Option<u32>,
    pub reroll_dice_below: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Critical {
    None,
    Success,
    Failure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollResult {
    pub total: i32,
    pub individual_rolls: Vec<u32>,
    pub critical: Critical,
    pub roll_used: RollPlan,
}

impl RollResult {
    pub fn is_critical_success(&self) -> bool {
        self.critical == Critical::Success
    }

    pub fn is_critical_failure(&self) -> bool {
        self.critical == Critical::Failure
    }

    pub fn meets_dc(&self, dc: i32) -> bool {
        match self.critical {
            Critical::Success => true,
            Critical::Failure => false,
            Critical::None => self.total >= dc,
        }
    }

    pub fn pretty_print(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "Rolled ")?;
        self.roll_used.pretty_print(f)?;
        write!(f, ": [")?;
        for (i, roll) in self.individual_rolls.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", roll)?;
        }
        write!(f, "] = {}", self.total)?;
        match self.critical {
            Critical::Success => write!(f, " (Critical Success)")?,
            Critical::Failure => write!(f, " (Critical Failure)")?,
            Critical::None => {}
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct RollPlan {
    pub num_dice: u32,
    pub die_size: u32,
    pub modifier: i32,
    pub settings: RollSettings,
}

impl RollPlan {
    pub fn roll(&self, rng: &mut Roller) -> anyhow::Result<RollResult> {
        match self.settings.advantage {
            Advantage::Normal => self.roll_normal(rng),
            Advantage::Advantage => self.roll_advantage(rng),
            Advantage::Disadvantage => self.roll_disadvantage(rng),
        }
    }

    fn roll_normal(&self, rng: &mut Roller) -> anyhow::Result<RollResult> {
        let low = self.settings.reroll_dice_below.unwrap_or(1);

        let clamp_min = self.settings.minimum_die_value.unwrap_or(1);
        let clamp_max = self.settings.maximum_die_value.unwrap_or(self.die_size);

        let mut individual_rolls = Vec::new();
        let mut total = 0;
        let mut critical = Critical::None;
        let mut crit_success_count = 0;
        let mut crit_failure_count = 0;

        for _ in 0..self.num_dice {
            let roll = rng.roll(low, self.die_size);
            let clamped_roll = roll.clamp(clamp_min, clamp_max);
            individual_rolls.push(clamped_roll);
            total += clamped_roll as i32;

            // crits can only happen on d20s
            if self.die_size == 20 {
                if clamped_roll == 20 {
                    crit_success_count += 1;
                } else if clamped_roll == 1 {
                    crit_failure_count += 1;
                }
            }
        }

        if crit_success_count > crit_failure_count {
            critical = Critical::Success;
        } else if crit_failure_count > crit_success_count {
            critical = Critical::Failure;
        }

        total += self.modifier;

        Ok(RollResult {
            total,
            individual_rolls,
            critical,
            roll_used: *self,
        })
    }

    fn roll_advantage(&self, rng: &mut Roller) -> anyhow::Result<RollResult> {
        let first_roll = self.roll_normal(rng)?;
        if first_roll.is_critical_success() {
            return Ok(first_roll);
        }

        let second_roll = self.roll_normal(rng)?;
        if second_roll.is_critical_success() {
            return Ok(second_roll);
        }

        if first_roll.total >= second_roll.total {
            Ok(first_roll)
        } else {
            Ok(second_roll)
        }
    }

    fn roll_disadvantage(&self, rng: &mut Roller) -> anyhow::Result<RollResult> {
        let first_roll = self.roll_normal(rng)?;
        if first_roll.is_critical_failure() {
            return Ok(first_roll);
        }

        let second_roll = self.roll_normal(rng)?;
        if second_roll.is_critical_failure() {
            return Ok(second_roll);
        }

        if first_roll.total <= second_roll.total {
            Ok(first_roll)
        } else {
            Ok(second_roll)
        }
    }

    pub fn pretty_print(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}d{}", self.num_dice, self.die_size)?;
        if self.modifier > 0 {
            write!(f, "+{}", self.modifier)?;
        } else if self.modifier < 0 {
            write!(f, "{}", self.modifier)?;
        }
        match self.settings.advantage {
            Advantage::Normal => {}
            Advantage::Advantage => write!(f, " adv")?,
            Advantage::Disadvantage => write!(f, " dis")?,
        }
        Ok(())
    }
}

impl From<&str> for RollPlan {
    fn from(value: &str) -> Self {
        crate::roll_parser::parse_roll(value).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roll() {
        let roll = RollPlan {
            num_dice: 2,
            die_size: 6,
            modifier: 3,
            settings: RollSettings {
                advantage: Advantage::Normal,
                minimum_die_value: None,
                maximum_die_value: None,
                reroll_dice_below: None,
            },
        };
        let mut rng = Roller::test_rng();
        for _ in 0..10000 {
            let result = roll.roll(&mut rng).unwrap();
            assert!(result.total >= 5 && result.total <= 15);
        }
    }

    #[test]
    fn test_roll_reroll_below() {
        let roll = RollPlan {
            num_dice: 1,
            die_size: 6,
            modifier: 0,
            settings: RollSettings {
                advantage: Advantage::Normal,
                minimum_die_value: None,
                maximum_die_value: None,
                reroll_dice_below: Some(3),
            },
        };
        let mut rng = Roller::test_rng();
        for _ in 0..10000 {
            let result = roll.roll(&mut rng).unwrap();
            assert!(result.total >= 3 && result.total <= 6);
        }
    }

    #[test]
    fn test_roll_min_max() {
        let roll = RollPlan {
            num_dice: 1,
            die_size: 6,
            modifier: 0,
            settings: RollSettings {
                advantage: Advantage::Normal,
                minimum_die_value: Some(3),
                maximum_die_value: Some(5),
                reroll_dice_below: None,
            },
        };
        let mut rng = Roller::test_rng();
        for _ in 0..10000 {
            let result = roll.roll(&mut rng).unwrap();
            assert!(result.total >= 3 && result.total <= 5);
        }
    }
}
