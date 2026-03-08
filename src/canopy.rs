use bevy::asset::RenderAssetUsages;
use bevy::math::Vec3;
use bevy::mesh::PrimitiveTopology;
use bevy::mesh::{Indices, Mesh};
use bevy::prelude::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct SphericalCap {
    pub radius: f32,
    pub depth: f32,
    pub radial_segments: u32,
    pub ring_count: u32,
}

impl Default for SphericalCap {
    fn default() -> Self {
        Self {
            radius: 0.15,
            depth: 0.06,
            radial_segments: 12,
            ring_count: 4,
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

impl From<SphericalCap> for Mesh {
    fn from(cap: SphericalCap) -> Self {
        assert!(cap.radius > 0.0, "Must have positive radius");
        assert!(cap.depth > 0.0, "Must have positive depth");
        assert!(
            cap.radial_segments > 2,
            "Must have at least 3 radial segments"
        );
        assert!(cap.ring_count > 0, "Must have at least 1 ring");

        let r = cap.radius;
        let d = cap.depth;
        // Sphere radius from rim radius and dome depth: R = (r² + d²) / (2d)
        let sphere_r = (r * r + d * d) / (2.0 * d);
        // Sphere center is below the apex
        let center_y = d - sphere_r;
        // Polar angle from apex to rim
        let theta_max = (r / sphere_r).asin();

        let n_verts = 1 + cap.ring_count * (cap.radial_segments + 1);
        let n_tris = cap.radial_segments + (cap.ring_count - 1) * cap.radial_segments * 2;
        let mut mesh = MeshData::new(n_verts as usize, (n_tris * 3) as usize);

        // Apex vertex
        mesh.positions.push(Vec3::new(0.0, d, 0.0));
        let apex_normal = Vec3::Y;
        mesh.normals.push(apex_normal);
        mesh.uvs.push(Vec2::new(0.5, 0.0));

        let tau = std::f32::consts::TAU;

        // Ring vertices (ring 1 nearest apex, ring_count at rim)
        for ring in 1..=cap.ring_count {
            let t = ring as f32 / cap.ring_count as f32;
            let theta = t * theta_max;
            let sin_theta = theta.sin();
            let cos_theta = theta.cos();
            let ring_r = sphere_r * sin_theta;
            let ring_y = center_y + sphere_r * cos_theta;

            for seg in 0..=cap.radial_segments {
                let phi = seg as f32 / cap.radial_segments as f32 * tau;
                let x = ring_r * phi.cos();
                let z = ring_r * phi.sin();

                mesh.positions.push(Vec3::new(x, ring_y, z));

                // Normal: outward from sphere center
                let normal = Vec3::new(x, ring_y - center_y, z).normalize();
                mesh.normals.push(normal);

                let u = seg as f32 / cap.radial_segments as f32;
                let v = t;
                mesh.uvs.push(Vec2::new(u, v));
            }
        }

        let segs = cap.radial_segments + 1; // verts per ring (including UV seam dup)

        // Triangle fan: apex to ring 1 (CCW winding for outward-facing normals)
        for seg in 0..cap.radial_segments {
            let ring1_base = 1;
            mesh.indices.push(0);
            mesh.indices.push(ring1_base + seg + 1);
            mesh.indices.push(ring1_base + seg);
        }

        // Quad strips between consecutive rings
        for ring in 0..(cap.ring_count - 1) {
            let cur_base = 1 + ring * segs;
            let next_base = 1 + (ring + 1) * segs;
            for seg in 0..cap.radial_segments {
                let tl = cur_base + seg;
                let tr = cur_base + seg + 1;
                let bl = next_base + seg;
                let br = next_base + seg + 1;

                mesh.indices.push(tl);
                mesh.indices.push(tr);
                mesh.indices.push(bl);

                mesh.indices.push(tr);
                mesh.indices.push(br);
                mesh.indices.push(bl);
            }
        }

        let mut m = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        m.insert_indices(Indices::U32(mesh.indices));
        m.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.positions);
        m.insert_attribute(Mesh::ATTRIBUTE_NORMAL, mesh.normals);
        m.insert_attribute(Mesh::ATTRIBUTE_UV_0, mesh.uvs);
        m
    }
}
