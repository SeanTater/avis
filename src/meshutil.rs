use bevy::prelude::*;

use crate::errors::*;

/// Estimate the normals of vertices by the mean of the normals of the faces they are in
pub fn estimate_vertex_normals(positions: &[Vec3], indices: &[u32]) -> Result<Vec<Vec3>> {
    let mut vertex_normals = vec![Vec3::ZERO; positions.len()];
    for verts in indices.chunks(3) {
        let v = [
            positions[verts[0] as usize],
            positions[verts[1] as usize],
            positions[verts[2] as usize],
        ];
        let a = v[1] - v[0];
        let b = v[2] - v[0];
        let face_normal = Vec3::from([
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ])
        .normalize_or_zero();
        for &vert in verts {
            vertex_normals[vert as usize] += face_normal;
        }
    }
    for v in vertex_normals.iter_mut() {
        *v = v.normalize_or_zero();
    }

    Ok(vertex_normals)
}
