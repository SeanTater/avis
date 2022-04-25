use std::sync::Weak;

use bevy::prelude::*;
use itertools::izip;

use crate::feature::Feature;

#[derive(Clone)]
pub struct Scatterplot {
    pub lats: Feature,
    pub lons: Feature,
    pub alts: Feature,
    pub sizes: Feature,
    pub colors: Vec<Color>,
}
#[derive(Component, Clone)]
pub struct ScatterplotPoint;
impl Scatterplot {
    fn setup_points(
        plot: Res<Scatterplot>,
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
    ) {
        for (lat, lon, alt, size, base_color) in izip!(
            plot.lats.convert(),
            plot.lons.convert(),
            plot.alts.convert(),
            plot.sizes.convert(),
            plot.colors.clone()
        ) {
            let material = materials.add(StandardMaterial {
                base_color,
                alpha_mode: AlphaMode::Blend,
                unlit: false,
                ..Default::default()
            });

            commands
                .spawn()
                .insert_bundle(PbrBundle {
                    mesh: meshes.add(
                        shape::Icosphere {
                            radius: size,
                            subdivisions: 3,
                        }
                        .into(),
                    ),
                    material,
                    transform: Transform::from_translation(Vec3::from([lon, alt, lat])),
                    ..Default::default()
                })
                .insert(ScatterplotPoint);
        }
    }
}
impl Plugin for Scatterplot {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.clone());
        app.add_startup_system(Scatterplot::setup_points);
    }
}
