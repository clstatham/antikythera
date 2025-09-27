pub fn factorial(n: u32) -> u32 {
    (1..=n).product()
}

pub fn multinomial_probability(
    n: u32,
    counts: &[u32],
    probabilities: &[f64],
) -> anyhow::Result<f64> {
    if counts.len() != probabilities.len() {
        anyhow::bail!("Counts and probabilities must have the same length");
    }
    if counts.iter().sum::<u32>() != n {
        anyhow::bail!("Counts must sum to n");
    }

    let numerator = factorial(n) as f64;
    let denominator: f64 = counts.iter().map(|&k| factorial(k) as f64).product();
    let prob_product: f64 = counts
        .iter()
        .zip(probabilities.iter())
        .map(|(&k, &p)| p.powi(k as i32))
        .product();
    Ok(numerator / denominator * prob_product)
}

pub fn binomial_coefficient(n: u32, k: u32) -> f64 {
    if k > n {
        return 0.0;
    }
    factorial(n) as f64 / (factorial(k) as f64 * factorial(n - k) as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
    }

    #[test]
    fn test_multinomial_probability() {
        let counts = vec![2, 1, 1];
        let probabilities = vec![0.5, 0.3, 0.2];
        let prob = multinomial_probability(4, &counts, &probabilities).unwrap();
        let expected = 12.0 * 0.5_f64.powi(2) * 0.3_f64.powi(1) * 0.2_f64.powi(1);
        assert!((prob - expected).abs() < 1e-6);
    }

    #[test]
    fn test_binomial_coefficient() {
        assert_eq!(binomial_coefficient(5, 2), 10.0);
        assert_eq!(binomial_coefficient(0, 0), 1.0);
        assert_eq!(binomial_coefficient(5, 0), 1.0);
        assert_eq!(binomial_coefficient(5, 5), 1.0);
        assert_eq!(binomial_coefficient(5, 6), 0.0);
    }
}
