use bevy::prelude::*;

/// A Triangle and custom UVs that can be converted to a Mesh.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct TriangleWithUvs {
    /// The Triangle data.
    pub triangle: Triangle3d,
    /// The UV data.
    pub uvs: [Vec2; 3],
}

impl From<TriangleWithUvs> for Mesh {
    fn from(TriangleWithUvs { triangle, uvs }: TriangleWithUvs) -> Self {
        let mesh = Mesh::from(triangle);

        mesh.with_inserted_attribute(
            Mesh::ATTRIBUTE_UV_0,
            uvs.map(Into::<[f32; 2]>::into).to_vec(),
        )
    }
}

impl TriangleWithUvs {
    /// Returns the centroid of the triangle and a new `TriangleWithUvs` whose vertices are
    /// relative to that centroid.
    pub fn centered(self) -> (Vec3, TriangleWithUvs) {
        let centroid = self.triangle.centroid();

        let triangle = Triangle3d {
            vertices: self.triangle.vertices.map(|vertex| vertex - centroid),
        };

        (centroid, TriangleWithUvs { triangle, ..self })
    }
}
