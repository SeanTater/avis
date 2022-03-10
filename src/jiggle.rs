use bevy::prelude::*;

pub struct JigglePlugin;
impl Plugin for JigglePlugin {
    fn build(&self, app: &mut App) {
        todo!()
    }
}

#[derive(Component)]
pub struct Jiggle {
    seed: f32,
    adjustment: Transform,
    next_frame: Transform,
    interval: f32,
    noise_factor: f32
}
fn update_jiggle(mut jiggles: Query<&mut Jiggle>) {
    for mut jiggle in jiggles.iter_mut() {
        jiggle.adjustment = jiggle.adjustment.mul_transform(jiggle.next_frame);
        jiggle.next_frame = Transform::from_scale(Vec3::splat(rand::random())).with_rotation(Quat::rand::random()).with_translation(rand::random());
        Vec3::from([rand::random::<f32>(), rand::random(), rand::random()]) * 0.01;
    }
}
fn apply_jiggle(mut transforms: Query<(&mut Transform, &Jiggle)>) {
    
    for (mut transform, jiggle) in transforms.iter_mut() {
        transform -= jiggle.adjustment
    }
}