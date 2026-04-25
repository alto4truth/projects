#[cfg(test)]
mod tests {
    use super::super::stats::*;

    #[test]
    fn test_mean() {
        let vals = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(mean(&vals), 2.5);
    }

    #[test]
    fn test_median_even() {
        let mut vals = [2.0, 1.0, 4.0, 3.0];
        assert_eq!(median(&mut vals), 2.5);
    }

    #[test]
    fn test_median_odd() {
        let mut vals = [5.0, 1.0, 3.0];
        assert_eq!(median(&mut vals), 3.0);
    }

    #[test]
    fn test_variance() {
        let vals = [1.0, 2.0, 3.0];
        assert!((variance(&vals) - 0.6666666667).abs() < 1e-6);
    }

    #[test]
    fn test_std_dev() {
        let vals = [1.0, 2.0, 3.0];
        assert!((std_dev(&vals) - 0.8164965809).abs() < 1e-6);
    }
}
