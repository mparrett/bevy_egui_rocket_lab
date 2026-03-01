use bevy::prelude::*;

use bevy::mesh::VertexAttributeValues;

pub fn update_mesh_uvs_for_number_of_tiles(mesh: &mut Mesh, number_of_tiles: (f32, f32)) {
    if let Some(VertexAttributeValues::Float32x2(uvs)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0) {
        for uv in uvs {
            uv[0] *= number_of_tiles.0;
            uv[1] *= number_of_tiles.1;
        }
    }
}
