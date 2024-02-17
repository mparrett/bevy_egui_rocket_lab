use bevy::prelude::*;

use bevy::render::mesh::VertexAttributeValues;

// https://github.com/bevyengine/bevy/pull/7406/files

// Update a mesh's UVs so that the applied texture tiles with the given `number_of_tiles`.
pub fn update_mesh_uvs_for_number_of_tiles(mesh: &mut Mesh, number_of_tiles: (f32, f32)) {
    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs {
            uv[0] *= number_of_tiles.0;
            uv[1] *= number_of_tiles.1;
        }
    }
}

// Update a quad's UVs so that the applied texture tiles with the calculated number of tiles,
// given the intended size of the texture in bevy units.
// pub fn update_quad_uvs_for_world_space_texture_size(
//     mesh: &mut Mesh,
//     world_space_texture_size: (f32, f32),
// ) {
//     let mut attributes = mesh.attributes_mut().collect::<HashMap<_, _>>();
//     if let (
//         Some(VertexAttributeValues::Float32x2(uvs)),
//         Some(VertexAttributeValues::Float32x3(positions)),
//     ) = (
//         attributes.remove(&Mesh::ATTRIBUTE_UV_0.id),
//         attributes.remove(&Mesh::ATTRIBUTE_POSITION.id),
//     ) {
//         for (position, uv) in positions.iter().zip(uvs) {
//             uv[0] = position[0] / world_space_texture_size.0;
//             // If you are using this to change the UVs of a shape::Plane, change this to -position[2]
//             //uv[1] = -position[1] / world_space_texture_size.1;
//             uv[1] = -position[2] / world_space_texture_size.1;
//         }
//     }
// }
