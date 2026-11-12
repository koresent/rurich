#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Measurement {
    pub minimum: usize,
    pub maximum: usize,
}

impl Measurement {
    pub const fn new(minimum: usize, maximum: usize) -> Self {
        Self { minimum, maximum }
    }

    pub fn span(&self) -> usize {
        self.maximum.saturating_sub(self.minimum)
    }

    pub fn normalize(&self) -> Self {
        let min = self.minimum.min(self.maximum);
        let max = self.maximum.max(min);
        Self {
            minimum: min,
            maximum: max,
        }
    }

    pub fn with_maximum(&self, width: usize) -> Self {
        Self {
            minimum: self.minimum.min(width),
            maximum: self.maximum.min(width),
        }
    }

    pub fn with_minimum(&self, width: usize) -> Self {
        Self {
            minimum: self.minimum.max(width),
            maximum: self.maximum.max(width),
        }
    }

    pub fn clamp(&self, min_width: Option<usize>, max_width: Option<usize>) -> Self {
        let mut m = *self;
        if let Some(min) = min_width {
            m = m.with_minimum(min);
        }
        if let Some(max) = max_width {
            m = m.with_maximum(max);
        }
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_measurement_clamp() {
        let m = Measurement::new(10, 50);
        assert_eq!(m.span(), 40);
        let clamped = m.clamp(Some(20), Some(40));
        assert_eq!(clamped.minimum, 20);
        assert_eq!(clamped.maximum, 40);
    }
}
