use range::{ AddTo, Range };
use wobj;
use wobj::obj::VTNIndex;
use std::default::Default;
use quack::{ Pair, SetAt };

use Position;
use TextureCoords;
use Normal;
use VertexFormat;
use VertexFormatError;

/// A geometry consists of a list of triangles.
/// The triangles are stored separately,
/// in a triangle list `u32` index buffer.
pub struct Geometry(pub Vec<Range>);

fn vtn_to_vertex<T>(a: VTNIndex, obj: &wobj::obj::Object) -> T
    where
        T: Default,
        (Position, T): Pair<Data = Position, Object = T> + SetAt,
        (TextureCoords, T): Pair<Data = TextureCoords, Object = T> + SetAt,
        (Normal, T): Pair<Data = Normal, Object = T> + SetAt
{
    use quack::Set;

    let mut vertex: T = Default::default();
    let position = obj.vertices[a.0];
    vertex.set_mut(Position([
        position.x as f32,
        position.y as f32,
        position.z as f32
    ]));
    if let Some(uv) = a.1 {
        let uv = obj.tex_vertices[uv];
        vertex.set_mut(TextureCoords([uv.x as f32, uv.y as f32]));
    }
    if let Some(normal) = a.2 {
        let normal = obj.normals[normal];
        vertex.set_mut(Normal([
            normal.x as f32,
            normal.y as f32,
            normal.z as f32
        ]));
    }
    vertex
}

impl Geometry {
    /// Creates a new geometry layer.
    pub fn new() -> Geometry {
        Geometry(Vec::new())
    }

    /// Adds geometry from Wavefront OBJ format to vertex and index buffer.
    pub fn add_geometry<T>(
        geom: &wobj::obj::Geometry,
        obj: &wobj::obj::Object,
        vertices: &mut Vec<T>,
        indices: &mut Vec<u32>
    ) -> (Range<AddTo<Geometry>>, Result<VertexFormat, VertexFormatError>)
        where
            T: Default,
            (Position, T): Pair<Data = Position, Object = T> + SetAt,
            (TextureCoords, T): Pair<Data = TextureCoords, Object = T> + SetAt,
            (Normal, T): Pair<Data = Normal, Object = T> + SetAt
    {
        let start = indices.len();
        let mut i = vertices.len() as u32;
        let mut uvs: u32 = 0;
        let mut normals: u32 = 0;
        {
            let mut add = |a: VTNIndex| {
                if let Some(_) = a.1 { uvs += 1; }
                if let Some(_) = a.2 { normals += 1; }
                vertices.push(vtn_to_vertex(a, obj));
                indices.push(i);
                i += 1;
            };
            for shape in geom.shapes.iter() {
                match shape {
                    // Extract triangles and offset them relative
                    // to the position in the index buffer.
                    &wobj::obj::Shape::Triangle(a, b, c) => {
                        add(a);
                        add(b);
                        add(c);
                    }
                    _ => {}
                }
            }
        }
        let n = indices.len() - start;
        let geometry = Range::new(start, n);
        let res = match (n as u32, uvs, normals) {
            (_, 0, 0) => { Ok(VertexFormat::Position) },
            (n, uvs, 0) if n == uvs => { Ok(VertexFormat::PositionTexture) }
            (n, 0, normals) if n == normals => {
                Ok(VertexFormat::PositionNormal)
            }
            (n, uvs, normals) if n == uvs && n == normals => {
                Ok(VertexFormat::PositionTextureNormal)
            }
            _ => Err(VertexFormatError::ExpectedSameVertexFormatPerGeometry)
        };
        (geometry, res)
    }
}

quack! {
    obj: Geometry[]
    get:
    set:
    action:
        fn (range: Range<AddTo<Geometry>>) -> () [] {
            obj.0.push(range.cast());
        }
}
