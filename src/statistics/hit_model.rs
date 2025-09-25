use crate::statistics::pmf::binomial_coefficient;

#[derive(Debug, Clone)]
pub struct HitModel {
    pub p_miss: f64,
    pub p_hit: f64,
    pub p_crit: f64,
}

impl HitModel {
    pub fn new(p_miss: f64, p_hit: f64, p_crit: f64) -> anyhow::Result<Self> {
        if (p_miss + p_hit + p_crit - 1.0).abs() > f64::EPSILON {
            anyhow::bail!("Probabilities must sum to 1");
        }
        Ok(HitModel {
            p_miss,
            p_hit,
            p_crit,
        })
    }

    /// Calculate the average damage per attack given the damage per hit and damage per crit.
    pub fn average_damage(&self, damage_per_hit: f64, damage_per_crit: f64) -> f64 {
        self.p_hit * damage_per_hit + self.p_crit * damage_per_crit
    }

    /// Calculate the probability of getting exactly `n_hits` hits and `n_crits` crits
    /// out of `n_attacks` attacks.
    pub fn probability(&self, n_attacks: u32, n_hits: u32, n_crits: u32) -> anyhow::Result<f64> {
        if n_crits > n_attacks || n_hits + n_crits > n_attacks {
            anyhow::bail!("Number of hits and crits cannot exceed number of attacks");
        }
        let n_misses = n_attacks - n_hits - n_crits;
        let coeff = binomial_coefficient(n_attacks, n_crits)
            * binomial_coefficient(n_attacks - n_crits, n_hits);
        let prob = coeff
            * self.p_crit.powi(n_crits as i32)
            * self.p_hit.powi(n_hits as i32)
            * self.p_miss.powi(n_misses as i32);
        Ok(prob)
    }
}

#[cfg(test)]
mod tests {
    use statrs::assert_almost_eq;

    use super::*;

    #[test]
    fn test_hit_model_new() {
        let model = HitModel::new(0.5, 0.4, 0.1).unwrap();
        assert_eq!(model.p_miss, 0.5);
        assert_eq!(model.p_hit, 0.4);
        assert_eq!(model.p_crit, 0.1);

        assert!(HitModel::new(0.5, 0.4, 0.2).is_err());
    }

    #[test]
    fn test_average_damage() {
        let model = HitModel::new(0.5, 0.4, 0.1).unwrap();
        let avg_damage = model.average_damage(10.0, 20.0);
        assert_almost_eq!(avg_damage, 6.0, f64::EPSILON);
    }

    #[test]
    fn test_probability() {
        let model = HitModel::new(0.5, 0.4, 0.1).unwrap();
        let prob = model.probability(3, 2, 1).unwrap();
        // There are 3 ways to arrange 2 hits and 1 crit in 3 attacks
        // Each arrangement has probability (0.4^2) * (0.1^1) * (0.5^0) = 0.016
        // Total probability = 3 * 0.016 = 0.048
        assert_almost_eq!(prob, 0.048, f64::EPSILON);

        assert!(model.probability(3, 4, 0).is_err());
        assert!(model.probability(3, 2, 2).is_err());
    }
}
