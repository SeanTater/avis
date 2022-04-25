use std::collections::BTreeMap;
use std::ops::RangeInclusive;

use anyhow::*;
use bevy::prelude::*;
use geo::map_coords::TryMapCoords;
use itertools::Itertools;

use crate::feature::{Overflow, Pipe};
use crate::meshutil::estimate_vertex_normals;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;

#[derive(Component)]
pub struct State {
    name: String,
    polygon: geo::Polygon<f32>,
}
impl State {
    /// Choose a color based on the hash of the name.
    /// The color will be a unit vector scaled to a range of 255 for each component.
    pub fn color(&self) -> Color {
        State::color_from_name(&self.name)
    }

    pub fn color_from_name(name: &str) -> Color {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        name.hash(&mut hasher);
        let color = hasher.finish().to_be_bytes();
        let mut color = [color[0] as f32, color[1] as f32, color[2] as f32];
        // Scale to a unit vector.
        let mag = color.iter().map(|x| x * x).sum::<f32>().sqrt();
        for i in 0..3 {
            color[i] /= mag;
        }
        Color::rgb(color[0], color[1], color[2])
    }
}
pub struct USMapPlugin;

#[derive(Component)]
pub struct USMap {
    lat_pipe: Pipe,
    lon_pipe: Pipe,
    loaded: bool,
}
impl USMap {
    pub fn new(lon_pipe: Pipe, lat_pipe: Pipe) -> Result<Self> {
        Ok(USMap {
            lat_pipe,
            lon_pipe,
            loaded: false,
        })
    }

    fn read_counties(&self) -> Result<Vec<State>> {
        // This JSON is invalid UTF8, unfortunately, so we fix it on the fly, which uses more memory.
        let shapes = std::fs::read("us-maps/geojson/state.geo.json")?;
        let shapes = String::from_utf8_lossy(&shapes)
            .into_owned()
            .parse::<geojson::GeoJson>()?;
        let mut counties = vec![];
        for feature in geojson::FeatureCollection::try_from(shapes)?.features {
            let name = feature
                .property("NAME10")
                .ok_or(anyhow!("State missing name"))?
                .as_str()
                .ok_or(anyhow!("State name is not a string"))?
                .to_string();
            if feature.geometry.is_none() {
                continue;
            }
            let polygons: geo::Geometry<f32> = feature.geometry.unwrap().try_into()?;
            let polygons = match polygons {
                geo::Geometry::MultiPolygon(mp) => mp.0,
                geo::Geometry::Polygon(p) => vec![p],
                _ => vec![],
            };
            for polygon in polygons {
                counties.push(State {
                    name: name.clone(),
                    polygon,
                })
            }
        }
        Ok(counties)
    }
}

impl Plugin for USMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_counties);
    }
}

/// Write a vector of vertices and indices to an obj file.
/// This is used to write the county meshes to disk.
/// The obj file can be loaded into Blender to visualize the mesh.
/// The obj file is not used in the game, but it is useful for debugging.
fn write_obj(vertices: &[geo::Point<f32>], indices: &[usize], filename: &str) -> Result<()> {
    use std::io::Write;
    let mut file = std::fs::File::create(filename)?;
    for vertex in vertices {
        writeln!(file, "v {} {} {}", vertex.x(), vertex.y(), 0.0)?;
    }
    for indexes in indices.chunks(3) {
        writeln!(
            file,
            "f {} {} {}",
            indexes[0] + 1,
            indexes[1] + 1,
            indexes[2] + 1
        )?;
    }
    Ok(())
}

/// Determine whether each angle is convex, assuming the points are in counter-clockwise order.
fn each_is_convex(points: &[geo::Point<f32>]) -> Vec<bool> {
    if points.len() <= 3 {
        return vec![true; points.len()];
    }
    // We start by calculating point one, not point zero. This is because for points A, B, C in that order,
    // the angle is A-B-C and we only know whether B is convex or not.
    // Fortunately, we only need to cheat a *little* to make this work, by using foreach and a mutable reference.
    let mut is_convex = vec![false; points.len()];
    points
        .iter()
        .circular_tuple_windows()
        .enumerate()
        .for_each(|(i, (&a, &b, &c))| {
            is_convex[(i+1) % points.len()] = b.cross_prod(a, c) < 0.0
        });
    is_convex
}
// Test is_convex on a complex shape.
#[test]
fn test_is_convex() {
    // This is a box with a dot in the middle.
    let points = [
        (0.0f32, 0.0f32),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.0, 1.0),
        (0.5, 0.5),
    ]
    .into_iter()
    .map(|(x, y)| geo::Point::new(x, y))
    .collect_vec();
    
    assert_eq!(each_is_convex(&points), vec![true, true, true, true, false]);

    // These are four points, where three are colinear, so it looks like a triangle,
    // but for our purposes is not considered convex because it will cause problems rendering.
    let points = [
        (0.0f32, 0.0f32),
        (1.0, 0.0),
        (1.0, 1.0),
        (0.5, 0.5),
    ]
    .into_iter()
    .map(|(x, y)| geo::Point::new(x, y))
    .collect_vec();
    assert_eq!(each_is_convex(&points), vec![true, true, true, false]);
}

/// Create a vector of triangles from a vector of points on the surface of a polygon.
/// The polygon may not be convex. Use ear clipping to create a convex polygon.
fn triangles_from_polygon(polygon: &geo::Polygon<f32>) -> Vec<[usize; 3]> {
    let mut points = polygon.exterior().points().collect_vec();
    points.pop(); // Remove the last point, which is a duplicate of the first.

    // If there are not at least three points, bail.
    if points.len() < 3 {
        return vec![];
    }

    // For each point in the polygon, find whether this angle is convex or concave.
    // Use a b-tree so that we can efficiently find the next point and still delete points in the middle.
    let mut is_convex = each_is_convex(&points).into_iter().enumerate().collect::<BTreeMap<_, _>>();

    // Print out how many points are convex and non convex, for debugging
    println!(
        "{} convex points before ear clipping",
        is_convex.values().filter(|&x| *x).count()
    );
    println!(
        "{} non-convex points before ear clipping",
        is_convex.values().filter(|&x| !*x).count()
    );

    //
    // Look for ears in the polygon, and create triangles from them.
    //
    let mut triangles = vec![];

    // search for (convex, convex, non-convex) angles to indicate an ear.
    // Then delete the ear by saving triangle ABC and deleting B, then recalculate the convexity of the angle ACD.
    let mut vertices_since_ear = 0;
    let mut i = 0;
    while is_convex.len() > 3  && vertices_since_ear < points.len() {
        let v = 
            (is_convex.range(i..))
            .chain(is_convex.range(0..i)).take(4)
            .map(|(x, y)| (*x, *y))
            .collect_vec();
        if v[0].1 && v[1].1 && !v[2].1 {
            // If the angle is convex, then we have an ear.
            // Save the triangle ABC and delete B.
            triangles.push([v[0].0, v[1].0, v[2].0]);
            is_convex.remove(&v[1].0);
            // Determine the new convexity of the angle ACD.
            let new_convex = points[v[2].0].cross_prod(points[v[0].0], points[v[3].0]) < 0.0;
            is_convex.insert(v[2].0, new_convex);
            vertices_since_ear = 0;
            // No change to i because we just deleted a point.
        } else {
            // Advance to the next point.
            i = v[1].0;
        }
        vertices_since_ear += 1;
    }
    // The remaining points form a convex polygon. First, let's verify that everything in is_convex is true.
    //assert!(is_convex.values().all(|x| *x), "Not all points are convex");
    // So we can use a basic algorithm to create triangles.
    // It's low quality but we'll create a simple fan of triangles from the first point.
    let mut keys = is_convex.into_keys();
    let middle = keys.next().unwrap();
    for (a, b) in keys.tuple_windows() {
        triangles.push([middle, a, b]);
    }

    triangles
}

/// Determine whether two triangles are equal, for tests.
/// The vertices must come the same order, but the array may be rotated.
fn triangles_equal(a: &[usize; 3], b: &[usize; 3]) -> bool {
    // Find the first element of a within b.
    let a_starts_at = match b.iter().find(|&&x| a[0] == x) {
        Some(i) => i,
        None => return false,
    };
    (0..3).all(|i| a[i] == b[(a_starts_at + i) % 3])
}

/// Test triangles_from_polygon on a basic unit square
/// This is used to verify that the algorithm works.
#[test]
fn test_triangles_from_polygon_on_a_square() {
    let polygon = geo::Polygon::new(
        geo::LineString::from(vec![
            geo::Point::new(0.0, 0.0),
            geo::Point::new(1.0, 0.0),
            geo::Point::new(1.0, 1.0),
            geo::Point::new(0.0, 1.0),
        ]),
        vec![],
    );
    let triangles = triangles_from_polygon(&polygon);
    assert_eq!(triangles.len(), 2);
    assert!(
        triangles_equal(&triangles[0], &[0, 1, 2]),
        "Triangle 0 is not correct: expected [0, 1, 2], got {:?}",
        triangles[0]
    );
    assert!(
        triangles_equal(&triangles[1], &[0, 2, 3]),
        "Triangle 1 is not correct: expected [0, 2, 3], got {:?}",
        triangles[1]
    );
}

/// Test triangles_from_polygon on a concave polygon
#[test]
fn test_triangles_from_polygon_on_a_concave_polygon() {
    let polygon = geo::Polygon::new(
        geo::LineString::from(vec![
            geo::Point::new(0.0, 0.0),
            geo::Point::new(1.0, 0.0),
            geo::Point::new(1.0, 1.0),
            geo::Point::new(0.0, 1.0),
            geo::Point::new(0.5, 0.5),
        ]),
        vec![],
    );
    let triangles = triangles_from_polygon(&polygon);
    assert_eq!(triangles.len(), 3);
    assert_eq!(triangles[0], [2, 3, 4]);
    assert_eq!(triangles[1], [1, 2, 4]);
    assert_eq!(triangles[2], [0, 1, 4]);
}

/// Set up a series of meshes to represent the counties
fn setup_counties(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut map: Query<&mut USMap>,
) {
    if map.get_single().is_err() {
        return;
    } else if map.single().loaded {
        return;
    } else {
        map.single_mut().loaded = true;
    }
    let map = map.single();
    for county in map.read_counties().expect("Failed to read counties") {
        // We could simplify the polygons to accelerate rendering, but it seems unnecessary.
        //.map(|poly| poly.simplifyvw_preserve(&0.0001))
        let poly = &county.polygon;
        let altitude = rand::random::<f32>() / 50.0;
        // let tris = delaunator::triangulate(
        //     &poly
        //         .exterior()
        //         .points()
        //         .map(|p| delaunator::Point {
        //             // These locations not displayed onscreen but for consistency we will scale them
        //             x: -map.lon_pipe.apply(p.x()) as f64,
        //             y: map.lat_pipe.apply(p.y()) as f64,
        //         })
        //         .collect::<Vec<_>>(),
        // );
        let tris = triangles_from_polygon(&poly)
            .into_iter()
            .flatten()
            .collect_vec();

        eprintln!(
            "{}, {} points, {} triangles",
            county.name,
            poly.exterior().points().count(),
            tris.len()
        );
        // write the obj file for debugging
        // write_obj(
        //     &poly.exterior().points().collect::<Vec<_>>(),
        //     &tris[..],
        //     &format!("us-maps/geojson/{}.obj", county.name),
        // ).expect("Failed to write obj file");

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let positions = poly
            .exterior()
            .points()
            .map(|p| {
                [
                    map.lon_pipe.apply(p.x()),
                    altitude,
                    map.lat_pipe.apply(p.y()),
                ]
            })
            .collect::<Vec<_>>();
        let uv0_pipe = Pipe::new(-180.0..=180.0, 0.0..=1.0).overflow(Overflow::Saturate);
        let uv0 = poly
            .exterior()
            .points()
            .map(|p| [uv0_pipe.apply(p.x()), uv0_pipe.apply(p.y())])
            .collect::<Vec<_>>();
        let indices = tris.into_iter().map(|u| u as u32).rev().collect::<Vec<_>>();
        let normals = estimate_vertex_normals(
            &positions.iter().map(|v| Vec3::from(*v)).collect::<Vec<_>>(),
            &indices,
        )
        .expect("Couldn't guess normals")
        .into_iter()
        .map(|v| v.to_array())
        .collect::<Vec<_>>();
        mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uv0);
        mesh.set_indices(Some(Indices::U32(indices)));

        let material = materials.add(StandardMaterial {
            base_color: county.color(),
            alpha_mode: AlphaMode::Blend,
            unlit: false,
            ..Default::default()
        });
        commands.spawn().insert(county).insert_bundle(PbrBundle {
            mesh: meshes.add(mesh),
            material,
            ..Default::default()
        });
    }
}
