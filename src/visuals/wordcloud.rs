use crate::errors::Result;

use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy_text_mesh::prelude::*;
use rand::prelude::*;
use serde::Deserialize;

// tessellation quality
const MESH_QUALITY: Quality = Quality::Low;
#[derive(Debug, Clone, Deserialize)]
struct WordParams {
    text: String,
    size: f32
}
#[derive(Debug, Clone, Deserialize)]
pub struct WordCloudVisual {
    words: Vec<WordParams> 
}
impl WordCloudVisual {
    pub fn new(word_info: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(word_info)?;
        Ok(serde_json::from_reader(file)?)
    }
    
    pub fn start(self) -> Result<()> {
        App::new()
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(self)
            .add_plugins(DefaultPlugins)
            .add_plugin(TextMeshPlugin)
            .add_startup_system(setup_background)
            .add_startup_system(setup_cloud)
            .add_system(rotate_camera)
            .add_system(lock_rotations)
            .run();
        Ok(())
    }
}

/// Shared data from the word cloud
struct Cloud {
    font: Handle<TextMeshFont>,
    material: Handle<StandardMaterial>,
}

/// Rotate this entity to always point to the camera
#[derive(Component)]
struct RotateLock;

/// Any text that is part of the word cloud
#[derive(Component)]
struct Word;

/// Some text that isn't part of the word cloud, a title
#[derive(Component)]
struct Legend;


impl Word {
    /// Add a word in any random direction
    fn add(
        commands: &mut Commands,
        font: &Handle<TextMeshFont>,
        material: &Handle<StandardMaterial>,
        text: &str,
        size: f32
    ) {
        let mut rng = rand::thread_rng();
        let transform = Transform {
            translation: Vec3::new(
                rng.gen_range(-1.0..1.0) * 2.0,
                rng.gen::<f32>() * 2.0,
                rng.gen_range(-1.0..1.0) * 2.0,
            ),
            scale: Vec3::ONE * size / 18.0,
            ..Default::default()
        }
        .looking_at(
            Vec3::new(rng.gen::<f32>(), rng.gen::<f32>(), rng.gen::<f32>()),
            Vec3::Y,
        );

        commands
            .spawn_bundle(TextMeshBundle {
                text_mesh: TextMesh {
                    text: text.into(),
                    style: TextMeshStyle {
                        font: font.clone(),
                        mesh_quality: MESH_QUALITY,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                transform,
                ..Default::default()
            })
            .insert(Word)
            .insert(RotateLock)
            .insert(material.clone());
    }
}

fn setup_cloud(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
    visual: Res<WordCloudVisual>,
    asset_server: Res<AssetServer>,
) {
    let state = Cloud {
        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
        material: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            unlit: true,
            ..Default::default()
        }),
    };

    commands
        .spawn_bundle(TextMeshBundle {
            text_mesh: TextMesh {
                text: String::from("Word Cloud Title"),
                style: TextMeshStyle {
                    font: state.font.clone(),
                    font_size: SizeUnit::NonStandard(18.),
                    color: Color::rgb(
                        0x09 as f32 / 256.0,
                        0x69 as f32 / 256.0,
                        0xda as f32 / 256.0,
                    ),
                    ..Default::default()
                },
                ..Default::default()
            },
            transform: Transform::from_xyz(0., 3., 0.),
            ..Default::default()
        })
        .insert(RotateLock)
        .insert(Legend);

    for word in &visual.words {
        Word::add(&mut commands, &state.font, &state.material, &word.text, word.size);
    }

    commands.insert_resource(state);
}

/// Orbit the center of the space, and rotate to face the center continuously
///
/// To make things a little more interesting, we'll make it elliptical
fn rotate_camera(mut camera: Query<&mut Transform, With<Camera>>, time: Res<Time>) {
    for mut camera in camera.iter_mut() {
        let sec = 0.2 * time.seconds_since_startup() as f32;
        *camera = Transform::from_xyz(5. * sec.sin(), 5., 10. * sec.cos())
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}

/// Keep the legend pointing at the camera all the time
fn lock_rotations(
    mut transform_pair: QuerySet<(
        QueryState<&Transform, With<Camera>>,
        QueryState<&mut Transform, With<RotateLock>>,
    )>,
) {
    let camera_translation = transform_pair.q0().single().translation;
    for mut locked_transform in transform_pair.q1().iter_mut() {
        // eh - why negative?
        *locked_transform = locked_transform.looking_at(-camera_translation, Vec3::Y);
    }
}

/// Create some context around the cloud
fn setup_background(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
