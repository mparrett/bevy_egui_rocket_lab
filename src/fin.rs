use bevy::math::Vec3;
use bevy::prelude::Vec2;
use bevy::mesh::{Indices, Mesh};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;

#[derive(Debug, Clone, Copy)]
pub struct Fin {
    pub height: f32,
    pub length: f32,
    pub width: f32,
}

impl Default for Fin {
    fn default() -> Self {
        Fin {
            height: 1.0,
            length: 0.5,
            width: 0.2,
        }
    }
}

struct MeshData {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    indices: Vec<u32>,
}

impl MeshData {
    fn new(num_vertices: usize, num_indices: usize) -> Self {
        Self {
            positions: Vec::with_capacity(num_vertices),
            normals: Vec::with_capacity(num_vertices),
            uvs: Vec::with_capacity(num_vertices),
            indices: Vec::with_capacity(num_indices),
        }
    }
}

fn create_fin_geometry(mesh: &mut MeshData, fin: &Fin) {
    let half_width = fin.width * 0.5;
    let height = fin.height;
    let length = fin.length;

    let front_bottom_left = Vec3::new(0.0, 0.0, -half_width);
    let front_top_left = Vec3::new(0.0, height, -half_width);
    let front_bottom_right = Vec3::new(length, 0.0, -half_width);

    let back_bottom_left = Vec3::new(0.0, 0.0, half_width);
    let back_top_left = Vec3::new(0.0, height, half_width);
    let back_bottom_right = Vec3::new(length, 0.0, half_width);

    mesh.positions.extend_from_slice(&[
        front_bottom_left,
        front_top_left,
        front_bottom_right,
        back_bottom_left,
        back_top_left,
        back_bottom_right,
    ]);

    mesh.uvs.extend_from_slice(&[
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 0.0),
    ]);

    mesh.indices.extend_from_slice(&[0, 1, 2]);
    mesh.indices.extend_from_slice(&[3, 5, 4]);

    mesh.indices.extend_from_slice(&[
        0, 3, 4, 0, 4, 1,
        1, 4, 5, 1, 5, 2,
        2, 5, 3, 2, 3, 0,
    ]);

    let _front_normal = (front_top_left - front_bottom_left)
        .cross(front_bottom_right - front_bottom_left)
        .normalize();
    let _back_normal = (back_bottom_right - back_bottom_left)
        .cross(back_top_left - back_bottom_left)
        .normalize();
    let left_side_normal = (back_top_left - front_top_left)
        .cross(front_bottom_left - front_top_left)
        .normalize();
    let right_side_normal = (front_bottom_right - back_bottom_right)
        .cross(back_top_left - back_bottom_right)
        .normalize();
    let bottom_normal = (front_bottom_right - front_bottom_left)
        .cross(back_bottom_left - front_bottom_left)
        .normalize();

    mesh.normals.extend_from_slice(&[
        bottom_normal,
        left_side_normal,
        right_side_normal,
        bottom_normal,
        left_side_normal,
        right_side_normal,
    ]);
}

impl From<Fin> for Mesh {
    fn from(fin: Fin) -> Self {
        let num_vertices = 6;
        let num_faces = 4;
        let num_indices = num_faces * 3;

        let mut mesh_data = MeshData::new(num_vertices, num_indices);

        create_fin_geometry(&mut mesh_data, &fin);

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );

        mesh.insert_indices(Indices::U32(mesh_data.indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh_data.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh_data.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh_data.uvs);

        mesh
    }
}
