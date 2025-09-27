use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct DeathSaves {
    pub successes: u8,
    pub failures: u8,
}

impl DeathSaves {
    pub fn record_success(&mut self) {
        if self.successes < 3 {
            self.successes += 1;
        }
    }

    pub fn record_failure(&mut self) {
        if self.failures < 3 {
            self.failures += 1;
        }
    }

    pub fn is_stable(&self) -> bool {
        self.successes >= 3
    }

    pub fn is_dead(&self) -> bool {
        self.failures >= 3
    }

    pub fn reset(&mut self) {
        self.successes = 0;
        self.failures = 0;
    }
}
