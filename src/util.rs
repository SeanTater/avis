use bevy::prelude::*;
pub fn rand_range(start: f32, end: f32) -> f32 {
    start + (end - start) * rand::random::<f32>()
}

pub trait Whatever {
    type Props: Clone;
    fn whatever(p: Self::Props) -> Self;
}

impl Whatever for Vec3 {
    type Props = ();
    fn whatever(_: Self::Props) -> Self {
        Vec3::from([rand::random(), rand::random(), rand::random()])
    }
}

impl Whatever for f32 {
    type Props = std::ops::Range<f32>;
    fn whatever(range: Self::Props) -> Self {
        (range.end - range.start) * (rand::random::<f32>() + range.start)
    }
}

impl<T: Whatever> Whatever for Vec<T> {
    type Props = (usize, T::Props);
    fn whatever((count, inner_p): Self::Props) -> Self {
        (0..count).map(|_| T::whatever(inner_p.clone())).collect()
    }
}
