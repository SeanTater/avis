use std::ops::RangeInclusive;

/// An f32 vector combined with a transformation to display it
#[derive(Debug, Clone)]
pub struct Feature {
    pub content: Vec<f32>,
    pub pipe: Pipe,
}
impl Feature {
    pub fn convert(&self) -> Vec<f32> {
        // These are usually tiny data, don't worry about inefficiency here
        self.content.iter().map(|v| self.pipe.apply(*v)).collect()
    }
}

/// A transformation from an input space to the display space (e.g. width of the Theater in meters)
#[derive(Debug, Clone)]
pub struct Pipe {
    domain: RangeInclusive<f32>,
    range: RangeInclusive<f32>,
    overflow: Overflow,
}

/// How should we handle out of bounds values? Clamp them, or allow them to overflow the range?
#[derive(Debug, Clone, Copy)]
pub enum Overflow {
    Extend,
    Saturate,
}

/// Create a pipe from a set of values as the domain
impl From<&[f32]> for Pipe {
    fn from(content: &[f32]) -> Self {
        Pipe::new(0.0..=1.0, 0.0..=1.0).infer_domain(content)
    }
}

impl Pipe {
    pub fn new(domain: RangeInclusive<f32>, range: RangeInclusive<f32>) -> Self {
        Pipe {
            domain,
            range,
            overflow: Overflow::Extend,
        }
    }
    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.overflow = overflow;
        self
    }
    pub fn infer_domain(mut self, content: &[f32]) -> Self {
        let min = content.iter().copied().reduce(f32::min).unwrap_or(f32::MIN);
        let mut max = content.iter().copied().reduce(f32::max).unwrap_or(f32::MAX);
        if min == max {
            // Avoid division by zero
            max = min + 1.0;
        }
        Pipe {
            domain: min..=max,
            range: min..=max,
            overflow: Overflow::Extend,
        }
    }
    pub fn set_domain(mut self, domain: &RangeInclusive<f32>) -> Self {
        self.domain = domain.clone();
        self
    }
    pub fn fit_to(mut self, range: &RangeInclusive<f32>) -> Self {
        self.range = range.clone();
        self
    }
    /// Apply this transformation to an f32
    pub fn apply(&self, mut val: f32) -> f32 {
        val -= self.domain.start();
        val /= self.domain.end() - self.domain.start();
        val *= self.range.end() - self.range.start();
        val += self.range.start();
        match self.overflow {
            Overflow::Saturate => val.clamp(*self.range.start(), *self.range.end()),
            Overflow::Extend => val,
        }
    }
    /// Bundle this pipe with a vector
    pub fn bundle(self, content: Vec<f32>) -> Feature {
        Feature {
            content,
            pipe: self,
        }
    }
}

#[test]
fn test_pipe_scale() {
    let p = Pipe::from(&[0.0, 1.0, 2.0][..]).fit_to(&(-2.0..=0.0));
    assert_eq!(p.apply(0.0), -2.0);
    assert_eq!(p.apply(1.0), -1.0);
    assert_eq!(p.apply(2.0), 0.0);
}

#[test]
fn test_pipe_overflow() {
    // Saturate
    let p = Pipe::from(&[0.0, 1.0, 2.0][..])
        .overflow(Overflow::Saturate)
        .set_domain(&(-1.0..=1.0))
        .fit_to(&(-2.0..=0.0));
    assert_eq!(p.apply(0.0), -1.0);
    assert_eq!(p.apply(1.0), 0.0);
    assert_eq!(p.apply(2.0), 0.0);

    // Default: Extend
    let p = Pipe::from(&[0.0, 1.0, 2.0][..])
        .set_domain(&(-1.0..=1.0))
        .fit_to(&(-2.0..=0.0));
    assert_eq!(p.apply(0.0), -1.0);
    assert_eq!(p.apply(1.0), 0.0);
    assert_eq!(p.apply(2.0), 1.0);
}
