use bevy::math::Vec3;
use bevy::prelude::Vec2;
use bevy::render::mesh::{Indices, Mesh};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;

#[derive(Debug, Clone, Copy)]
pub struct Fin {
    pub height: f32, // Height of the fin
    pub length: f32, // Length of the fin
    pub width: f32,  // Thickness, orthogonal to height and length
}

impl Default for Fin {
    fn default() -> Self {
        // Default for a fin that is 2x taller than it is long, and is 0.2 wide
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
    // Each fin is a right triangle with a thickness (width) extending radially from the rocket
    // The fin is defined by its height, length, and width
    //
    //   +-
    //   |\\
    // h | \\ w
    //   |  \\
    //   +---++
    //     l
    let half_width = fin.width * 0.5; // Half of the fin's thickness, adding depth
    let height = fin.height; // The height of the fin
    let length = fin.length; // The length of the fin extending radially

    // Vertices for the right triangle (front face)
    let front_bottom_left = Vec3::new(0.0, 0.0, -half_width); // Bottom left point
    let front_top_left = Vec3::new(0.0, height, -half_width); // Top left point, making the right angle
    let front_bottom_right = Vec3::new(length, 0.0, -half_width); // Bottom right point

    // Vertices for the right triangle (back face, mirrored along the Z-axis)
    let back_bottom_left = Vec3::new(0.0, 0.0, half_width); // Mirrored bottom left point
    let back_top_left = Vec3::new(0.0, height, half_width); // Mirrored top left point
    let back_bottom_right = Vec3::new(length, 0.0, half_width); // Mirrored bottom right point

    mesh.positions.extend_from_slice(&[
        front_bottom_left,
        front_top_left,
        front_bottom_right,
        back_bottom_left,
        back_top_left,
        back_bottom_right,
    ]);

    // UVs (simplified for this example)
    mesh.uvs.extend_from_slice(&[
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 0.0),
        Vec2::new(0.0, 0.0),
        Vec2::new(0.0, 1.0),
        Vec2::new(1.0, 0.0),
    ]);

    // Indices to form the triangles for each face
    mesh.indices.extend_from_slice(&[0, 1, 2]); // Front face
    mesh.indices.extend_from_slice(&[3, 5, 4]); // Back face

    // Side faces to create thickness (width) for the fin
    mesh.indices.extend_from_slice(&[
        0, 3, 4, 0, 4, 1, // Left side
        1, 4, 5, 1, 5, 2, // Diagonal side
        2, 5, 3, 2, 3, 0, // Bottom side
    ]);

    // Calculate normals
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
        bottom_normal,     // front_bottom_left
        left_side_normal,  // front_top_left
        right_side_normal, // front_bottom_right
        bottom_normal,     // back_bottom_left
        left_side_normal,  // back_top_left
        right_side_normal, // back_bottom_right
    ]);
}

impl From<Fin> for Mesh {
    fn from(fin: Fin) -> Self {
        let num_vertices = 6; // 3 vertices for each face, front and back
        let num_faces = 4; // 2 for the front and back, 2 for the sides, bottom is created by the sides
        let num_indices = num_faces * 3; // 3 indices per triangle

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
