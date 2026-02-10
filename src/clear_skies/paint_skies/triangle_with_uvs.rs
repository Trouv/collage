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

/// An octahedron type, built by creating 6 triangles between two triangular faces.
pub struct OctahedronWithUvs {
    /// The near face.
    pub near_face: TriangleWithUvs,
    /// The far face.
    pub far_face: TriangleWithUvs,
}

impl OctahedronWithUvs {
    /// Returns the centroid of the octahedron and a new `OctahedronWithUvs` whose vertices are
    /// relative to that centroid.,
    pub fn centered(self) -> (Vec3, OctahedronWithUvs) {
        let near_centroid = self.near_face.triangle.centroid();
        let far_centroid = self.far_face.triangle.centroid();

        let centroid = (near_centroid + far_centroid) / 2.0;

        let near_triangle = Triangle3d {
            vertices: self
                .near_face
                .triangle
                .vertices
                .map(|vertex| vertex - centroid),
        };
        let far_triangle = Triangle3d {
            vertices: self
                .far_face
                .triangle
                .vertices
                .map(|vertex| vertex - centroid),
        };

        let centered = OctahedronWithUvs {
            near_face: TriangleWithUvs {
                triangle: near_triangle,
                ..self.near_face
            },
            far_face: TriangleWithUvs {
                triangle: far_triangle,
                ..self.far_face
            },
        };

        (centroid, centered)
    }
}
