use crate::errors::Result;

use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy_text_mesh::prelude::*;
use rand::prelude::*;
use serde::Deserialize;

lazy_static::lazy_static! {
    static ref PRIMARY_COLOR: Color = Color::rgb_u8(1, 33, 105);
}

/// One row about a word
#[derive(Debug, Clone, Deserialize)]
struct WordParams {
    text: String,
    size: f32,
    category: String
}

/// All the configuration for a cloud, readable as a JSON file
#[derive(Debug, Clone, Deserialize)]
pub struct WordCloudVisual {
    title: String,
    words: Vec<WordParams> 
}
impl WordCloudVisual {
    pub fn new(word_info: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(word_info)?;
        Ok(serde_json::from_reader(file)?)
    }
    
    pub fn start(self) -> Result<()> {
        App::new()
            .insert_resource(bevy_atmosphere::AtmosphereMat::default()) 
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(self)
            .add_plugins(DefaultPlugins)
            .add_plugin(TextMeshPlugin)
            .add_plugin(bevy_atmosphere::AtmospherePlugin { dynamic: false })
            .add_plugin(bevy_fly_camera::FlyCameraPlugin)
            .add_startup_system(setup_background)
            .add_startup_system(setup_cloud)
            .add_system(lock_rotations)
            .add_system(scoot_words)
            .run();
        Ok(())
    }
}

/// Shared data from the word cloud
struct CloudState {
    font: Handle<TextMeshFont>,
    category: Option<String>
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
        materials: &mut Assets<StandardMaterial>,
        font: &Handle<TextMeshFont>,
        text: &str,
        size: f32
    ) {
        let mut rng = rand::thread_rng();
        let transform = Transform {
            translation: Vec3::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(0.0..4.0),
                rng.gen_range(-2.0..2.0),
            ),
            scale: Vec3::ONE * size.exp().sqrt() / 500.0,
            ..Default::default()
        };
        let color = fasthash::city::hash32(text.as_bytes()).to_be_bytes();
        let color = Color::rgb_u8(color[0], color[1], color[2]);
        let material = materials.add(StandardMaterial {
            base_color: color,
            emissive: Color::DARK_GRAY,
            ..Default::default()
        });


        commands
            .spawn_bundle(TextMeshBundle {
                text_mesh: TextMesh {
                    text: text.into(),
                    style: TextMeshStyle {
                        font: font.clone(),
                        color,
                        ..Default::default()
                    },
                    size: TextMeshSize {
                        wrapping: false,
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
    let state = CloudState {
        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..Default::default()
        }),
        // Any category
        category: Some(visual.words[rand::random::<usize>() % visual.words.len()].category.clone())
    };

    commands
        .spawn_bundle(TextMeshBundle {
            text_mesh: TextMesh {
                text: visual.title.clone(),
                style: TextMeshStyle {
                    font: state.font.clone(),
                    font_size: SizeUnit::NonStandard(18.),
                    color: *PRIMARY_COLOR,
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
        if Some(&word.category) == state.category.as_ref() {
            Word::add(&mut commands,  materials.as_mut(), &state.font, word.text.trim(), word.size);
        }
    }

    commands.insert_resource(state);
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

/// Space the words better
fn scoot_words(
    mut transforms: Query<(&mut Transform, &TextMesh)>,
) {
    let mut combo_iter = transforms.iter_combinations_mut();
    while let Some([(left_transform, left_textmesh), (mut right_transform, right_textmesh)]) = combo_iter.fetch_next() {
        let distance: f32 = left_transform.translation.distance(right_transform.translation);
        let needed_distance: f32 = (
            left_textmesh.text.len()
            + right_textmesh.text.len()
        ) as f32 / 500.0;
        let push = 1.0 + (needed_distance - distance).tanh();
        let push_vector = (right_transform.translation - left_transform.translation) * Vec3::from([1.0, 0.0, 1.0]) * 0.01 * push;
        right_transform.translation = right_transform.translation + push_vector;
    }
}

/// Create some context around the cloud
fn setup_background(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
        material: materials.add( StandardMaterial {
            perceptual_roughness: 0.5,
            base_color_texture: Some(asset_server.load("textures/wood_floor.jpg")),
            ..Default::default()
        }),
        transform: Transform::from_xyz(0.0, -1.0, 0.0),
        ..Default::default()
    });
    commands.insert_resource(AmbientLight {
        brightness: 0.6,
        ..Default::default()
    });
    
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            color: Color::ANTIQUE_WHITE,
            ..Default::default()
        },
        ..Default::default()
    });

    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 3.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    })
    .insert(bevy_fly_camera::FlyCamera::default());
}
