use bevy::prelude::*;
use std::{f32::consts::PI, ops::RangeInclusive};

pub struct Theater {
    pub width: RangeInclusive<f32>,
    pub height: RangeInclusive<f32>,
    pub depth: RangeInclusive<f32>,
}
impl Default for Theater {
    fn default() -> Self {
        Theater {
            width: -5.0..=5.0,
            height: 0.0..=5.0,
            depth: -5.0..=5.0, 
        }
    }
}
impl Plugin for Theater {
    fn build(&self, app: &mut App) {
        app.insert_resource(bevy_atmosphere::AtmosphereMat::default())
            .insert_resource(Msaa { samples: 4 })
            .add_plugins(DefaultPlugins)
            .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false })
            .add_plugin(bevy_fly_camera::FlyCameraPlugin)
            .add_startup_system(setup_world);
    }
}

fn setup_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor, if there is any meaning to it.
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(bevy::prelude::shape::Plane { size: 100. }.into()),
        material: materials.add(StandardMaterial {
            base_color: Color::ANTIQUE_WHITE,
            ..Default::default()
        }),
        ..Default::default()
    });

    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 10000.,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 8.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..Default::default()
        },
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        brightness: 0.25,
        ..Default::default()
    });

    // Flyable camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(1.0, 2.0, 4.0)
                .looking_at(Vec3::from((10.0, 0.0, -10.)), Vec3::Y),
            ..Default::default()
        })
        .insert(bevy_fly_camera::FlyCamera::default());
}
