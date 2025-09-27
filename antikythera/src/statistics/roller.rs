use rand::{Rng, SeedableRng, rngs::StdRng};

#[derive(Debug)]
pub struct Roller {
    rng: StdRng,
}

impl Roller {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        let rng = StdRng::from_os_rng();
        Roller { rng }
    }

    /// Creates a new `Roller` instance with a different random seed.
    /// Useful for creating independent random number generators in multi-threaded contexts.
    pub fn fork(&mut self) -> Self {
        let mut seed = [0u8; 32];
        self.rng.fill(&mut seed);
        let rng = StdRng::from_seed(seed);
        Roller { rng }
    }

    pub fn from_seed(seed: u64) -> Self {
        let rng = StdRng::seed_from_u64(seed);
        Roller { rng }
    }

    pub fn d(&mut self, die_size: u32) -> u32 {
        self.rng.random_range(1..=die_size)
    }

    pub fn roll(&mut self, min: u32, max: u32) -> u32 {
        self.rng.random_range(min..=max)
    }

    pub fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    #[cfg(test)]
    pub fn test_rng() -> Self {
        Self::from_seed(42)
    }
}
