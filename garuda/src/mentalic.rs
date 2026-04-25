#[derive(Debug, Clone, PartialEq)]
pub struct MentalState {
    pub focus: f64,
    pub load: f64,
    pub confidence: f64,
}

impl MentalState {
    pub fn new(focus: f64, load: f64, confidence: f64) -> Self {
        Self {
            focus: focus.clamp(0.0, 1.0),
            load: load.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }

    pub fn score(&self) -> f64 {
        (self.focus * 0.5) + ((1.0 - self.load) * 0.25) + (self.confidence * 0.25)
    }
}

pub fn stabilize(states: &[MentalState]) -> MentalState {
    if states.is_empty() {
        return MentalState::new(0.0, 0.0, 0.0);
    }

    let count = states.len() as f64;
    let focus = states.iter().map(|state| state.focus).sum::<f64>() / count;
    let load = states.iter().map(|state| state.load).sum::<f64>() / count;
    let confidence = states.iter().map(|state| state.confidence).sum::<f64>() / count;

    MentalState::new(focus, load, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_inputs() {
        let state = MentalState::new(2.0, -1.0, 0.5);
        assert_eq!(state.focus, 1.0);
        assert_eq!(state.load, 0.0);
        assert_eq!(state.confidence, 0.5);
    }

    #[test]
    fn stabilizes_multiple_states() {
        let merged = stabilize(&[
            MentalState::new(1.0, 0.2, 0.9),
            MentalState::new(0.5, 0.4, 0.7),
        ]);
        assert!(merged.focus > 0.7 && merged.focus < 0.8);
        assert!(merged.load > 0.2 && merged.load < 0.4);
    }
}
