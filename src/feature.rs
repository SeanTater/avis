use std::ops::RangeInclusive;

#[derive(Debug, Clone)]
pub struct Feature {
    content: Vec<f32>,
    domain: RangeInclusive<f32>,
    range: RangeInclusive<f32>,
    overflow: Overflow,
}
#[derive(Debug, Clone, Copy)]
pub enum Overflow {
    Extend,
    Saturate
}
impl From<Vec<f32>> for Feature {
    fn from(content: Vec<f32>) -> Self {
        let min = content.iter().copied().reduce(f32::min).unwrap_or(f32::MIN);
        let mut max = content.iter().copied().reduce(f32::max).unwrap_or(f32::MAX);
        if min == max {
            // Avoid division by zero
            max = min + 1.0;
        }
        Feature {
            content,
            domain: min..=max,
            range: min..=max,
            overflow: Overflow::Extend,
        }
    }
}
impl Feature {
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }
    pub fn set_domain(mut self, domain: &RangeInclusive<f32>) -> Self {
        self.domain = domain.clone();
        self
    }
    pub fn fit_to(mut self, range: &RangeInclusive<f32>) -> Self {
        self.range = range.clone();
        self
    }
    pub fn convert(&self) -> Vec<f32> {
        // These are usually tiny data, don't worry about inefficiency here
        self.content.iter().map(|val| {
            let mut val = *val;
            val -= self.domain.start();
            val /= self.domain.end() - self.domain.start();
            val *= self.range.end() - self.range.start();
            val += self.range.start();
            match self.overflow {
                Overflow::Saturate => val.clamp(*self.range.start(), *self.range.end()),
                Overflow::Extend => val
            }
        }).collect()
    }
}

#[test]
fn test_feature_scale() {
    assert_eq!(
        Feature::from(vec![0.0, 1.0, 2.0]).fit_to(&(-2.0..=0.0)).convert(),
        vec![-2.0, -1.0, 0.0]
    );
}

#[test]
fn test_feature_overflow() {
    // Saturate
    assert_eq!(
        Feature::from(vec![0.0, 1.0, 2.0])
            .overflow(Overflow::Saturate)
            .set_domain(&(-1.0..=1.0))
            .fit_to(&(-2.0..=0.0))
            .convert(),
        vec![-1.0, 0.0, 0.0]
    );
    // Default: Extend
    assert_eq!(
        Feature::from(vec![0.0, 1.0, 2.0])
            .set_domain(&(-1.0..=1.0))
            .fit_to(&(-2.0..=0.0))
            .convert(),
        vec![-1.0, 0.0, 1.0]
    );
}