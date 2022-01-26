use std::f32::consts::PI;

use bevy::prelude::*;

use crate::util::rand_range;

pub struct People;

#[derive(Component)]
pub struct Person {
    speed: f32,
    intent: Intent,
    role: Role,
}

#[derive(Clone, Copy)]
enum Role {
    Associate,
    Customer,
}

#[derive(Clone, Copy)]
enum Intent {
    Walking(Vec3),
    Waiting(f32),
    Idle
}

impl Plugin for People {
    fn build(&self, app: &mut App) {
        app.add_startup_system(add_people)
            .add_system(update_people)
            .add_system(animate_people);
    }
}

/// Pick any spot on the floor
fn somewhere() -> Vec3 {
    Vec3::from([rand_range(-50., 50.), 0.0, rand_range(-50., 50.)])
}

/// Add a person to the simulation
fn add_people(mut commands: Commands, assets: Res<AssetServer>) {
    let associate = assets.load("models/person/associate.glb#Scene0");
    let customer = assets.load("models/person/customer.glb#Scene0");
    for _ in 0..50 {
        let transform = Transform::from_translation(somewhere())
            .with_rotation(Quat::from_axis_angle(Vec3::Y, rand_range(-PI, PI)));
        let role = if rand::random() {
            Role::Associate
        } else {
            Role::Customer
        };
        commands
            .spawn_bundle((
                Person {
                    speed: 10.0,
                    intent: Intent::Idle,
                    role,
                },
                transform,
                GlobalTransform::identity(),
            ))
            .with_children(|parent| {
                match role {
                    Role::Associate => parent.spawn_scene(associate.clone()),
                    Role::Customer => parent.spawn_scene(customer.clone()),
                };
            });
    }
}

/// Choose where a person wants to go next
fn update_people(mut people: Query<(&Transform, &mut Person)>, time: Res<Time>) {
    for (transform, mut person) in people.iter_mut() {
        person.intent = match person.intent {
            current @ Intent::Walking(target) => {
                if target.distance(transform.translation) < 1. {
                    Intent::Idle
                } else {
                    current
                }
            },
            Intent::Waiting(seconds) => {
                if seconds <= 0.0 {
                    Intent::Idle
                } else {
                    Intent::Waiting(seconds - time.delta_seconds())
                }
            },
            Intent::Idle => {
                match rand::random() {
                    0.0..=0.8 => Intent::Waiting(1.0),
                    0.8..=1.0 => Intent::Walking(somewhere()),
                    _ => unreachable!()
                }
            }
        };
    }
}

/// Animate people to move toward their targets
fn animate_people(mut transforms: Query<(&mut Transform, &Person)>, time: Res<Time>) {
    for (mut transform, person) in transforms.iter_mut() {
        if let Intent::Walking(target) = person.intent {
            let perfect_direction = transform.clone().looking_at(target, Vec3::Y);
            // Turn in about 1/2 a second (it approaches exponentially)
            let turn_speed = 2.0;
            transform.rotation = transform
                .rotation
                .lerp(perfect_direction.rotation, (time.delta_seconds() * turn_speed).min(0.25));
            // Move only forward, by a steady amount
            transform.translation =
                transform.translation + transform.forward() * person.speed * time.delta_seconds();
        }
    }
}
