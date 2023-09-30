#![warn(clippy::pedantic)]

use std::num::{ParseFloatError, ParseIntError};

#[derive(Debug)]
pub enum OBJParseError {
    MissingCoordinate,
    MissingPrefix,
    CoordinateParseError(ParseFloatError),
    WeightParseError(ParseFloatError),
    NotEnoughTripletsInFace,
    MissingVertexInTriplet,
    TripletComponentParseError(ParseIntError),
}

pub struct OBJParser;
impl OBJParser {
    /// # Errors
    /// This function may fail if the file is invalid
    #[allow(clippy::needless_pass_by_value)]
    pub fn parse<S: ToString>(obj_content: S) -> Result<OBJModel, OBJParseError> {
        let obj_content = obj_content.to_string();
        let mut model = OBJModel::default();
        for line in obj_content.lines() {
            let mut split_line = line.split(' ');
            let prefix = split_line.next().ok_or(OBJParseError::MissingPrefix)?;
            match prefix {
                "v" => model
                    .geometric_vertices
                    .push(parse_geometric_vertex(split_line)?),
                "vn" => model.vertex_normals.push(parse_vertex_normal(split_line)?),
                "vt" => model
                    .texture_vertices
                    .push(parse_texture_vertex(split_line)?),
                "f" => model.faces.push(parse_face(split_line)?),
                _ => {}
            }
        }

        Ok(model)
    }
}

fn parse_vertex_normal(
    mut split_line: std::str::Split<char>,
) -> Result<VertexNormal, OBJParseError> {
    let i_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let i = parse_coordinate(i_str)?;
    let j_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let j = parse_coordinate(j_str)?;
    let k_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let k = parse_coordinate(k_str)?;

    Ok(VertexNormal { i, j, k })
}

fn parse_geometric_vertex(
    mut split_line: std::str::Split<char>,
) -> Result<GeometricVertex, OBJParseError> {
    let x_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let x = parse_coordinate(x_str)?;
    let y_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let y = parse_coordinate(y_str)?;
    let z_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let z = parse_coordinate(z_str)?;

    let w_str = split_line.next();
    let w = if let Some(w_str) = w_str {
        parse_weight(w_str)?
    } else {
        1.0
    };

    Ok(GeometricVertex { x, y, z, w })
}

fn parse_texture_vertex(
    mut split_line: std::str::Split<char>,
) -> Result<TextureVertex, OBJParseError> {
    let u_str = split_line.next().ok_or(OBJParseError::MissingCoordinate)?;
    let u = parse_coordinate(u_str)?;

    let v_str = split_line.next();
    let v = if let Some(v_str) = v_str {
        1.0 - parse_coordinate(v_str)?
    } else {
        0.0
    };

    let w_str = split_line.next();
    let w = if let Some(w_str) = w_str {
        parse_coordinate(w_str)?
    } else {
        0.0
    };

    Ok(TextureVertex { u, v, w })
}

fn parse_face(split_line: std::str::Split<char>) -> Result<Face, OBJParseError> {
    let mut triplets = vec![];
    for triplet_str in split_line {
        triplets.push(parse_triplet(triplet_str)?);
    }

    if triplets.len() < 3 {
        return Err(OBJParseError::NotEnoughTripletsInFace);
    }

    Ok(Face { triplets })
}

fn parse_triplet(triplet_str: &str) -> Result<Triplet, OBJParseError> {
    let mut split_triplet = triplet_str.split('/');

    let geometric_vertex_str = split_triplet
        .next()
        .ok_or(OBJParseError::MissingVertexInTriplet)?;
    let geometric_vertex = geometric_vertex_str
        .parse()
        .map_err(OBJParseError::TripletComponentParseError)?;

    let texture_vertex_str = split_triplet.next();
    let texture_vertex = if let Some(texture_vertex_str) = texture_vertex_str {
        if texture_vertex_str.is_empty() {
            None
        } else {
            Some(
                texture_vertex_str
                    .parse()
                    .map_err(OBJParseError::TripletComponentParseError)?,
            )
        }
    } else {
        None
    };

    let vertex_normal_str = split_triplet.next();
    let vertex_normal = if let Some(vertex_normal_str) = vertex_normal_str {
        if vertex_normal_str.is_empty() {
            None
        } else {
            Some(
                vertex_normal_str
                    .parse()
                    .map_err(OBJParseError::TripletComponentParseError)?,
            )
        }
    } else {
        None
    };

    Ok(Triplet {
        geometric_vertex,
        texture_vertex,
        vertex_normal,
    })
}

fn parse_coordinate(coordinate_str: &str) -> Result<f32, OBJParseError> {
    coordinate_str
        .parse()
        .map_err(OBJParseError::CoordinateParseError)
}

fn parse_weight(weight_str: &str) -> Result<f32, OBJParseError> {
    weight_str
        .parse::<f32>()
        .map_err(OBJParseError::WeightParseError)
}

#[derive(Default, Debug, Clone)]
pub struct OBJModel {
    pub geometric_vertices: Vec<GeometricVertex>,
    pub texture_vertices: Vec<TextureVertex>,
    pub vertex_normals: Vec<VertexNormal>,
    pub faces: Vec<Face>,
}

#[derive(Debug, Clone)]
pub struct GeometricVertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Debug, Clone)]
pub struct VertexNormal {
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

#[derive(Debug, Clone)]
pub struct TextureVertex {
    pub u: f32,
    pub v: f32,
    pub w: f32,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub triplets: Vec<Triplet>,
}

#[derive(Debug, Clone)]
pub struct Triplet {
    pub geometric_vertex: usize,
    pub texture_vertex: Option<usize>,
    pub vertex_normal: Option<usize>,
}

#[cfg(test)]
#[allow(clippy::cast_possible_truncation)]
mod tests {
    use super::*;

    #[test]
    fn parse_geometric_vertex() {
        let obj_model = OBJParser::parse(
            r#"
v 1.0 2.0 3.0
        "#,
        )
        .unwrap();

        assert_eq!(obj_model.geometric_vertices.len(), 1);

        let vertex = &obj_model.geometric_vertices[0];
        assert_eq!(vertex.x as i32, 1);
        assert_eq!(vertex.y as i32, 2);
        assert_eq!(vertex.z as i32, 3);
        assert_eq!(vertex.w as i32, 1);
    }

    #[test]
    fn parse_texture_vertex() {
        let obj_model = OBJParser::parse(
            r#"
vt 16.0 32.0
        "#,
        )
        .unwrap();

        assert_eq!(obj_model.texture_vertices.len(), 1);

        let texture_vertex = &obj_model.texture_vertices[0];
        assert_eq!(texture_vertex.u as i32, 16);
        assert_eq!(texture_vertex.v as i32, 32);
        assert_eq!(texture_vertex.w as i32, 0);
    }

    #[test]
    fn parse_vertex_normal() {
        let obj_model = OBJParser::parse(
            r#"
vn 2.0 4.0 1.0
        "#,
        )
        .unwrap();

        assert_eq!(obj_model.vertex_normals.len(), 1);

        let vertex_normal = &obj_model.vertex_normals[0];
        assert_eq!(vertex_normal.i as i32, 2);
        assert_eq!(vertex_normal.j as i32, 4);
        assert_eq!(vertex_normal.k as i32, 1);
    }

    #[test]
    fn parse_model() {
        let obj_model = OBJParser::parse(
            r#"o Cube
v 0.500000 0.500000 -0.500000
v 0.500000 -0.500000 -0.500000
v 0.500000 0.500000 0.500000
v 0.500000 -0.500000 0.500000
v -0.500000 0.500000 -0.500000
v -0.500000 -0.500000 -0.500000
v -0.500000 0.500000 0.500000
v -0.500000 -0.500000 0.500000
vt 0.625000 0.500000
vt 0.375000 0.500000
vt 0.625000 0.750000
vt 0.375000 0.750000
vt 0.875000 0.500000
vt 0.625000 0.250000
vt 0.125000 0.500000
vt 0.375000 0.250000
vt 0.875000 0.750000
vt 0.625000 1.000000
vt 0.625000 0.000000
vt 0.375000 0.000000
vt 0.375000 1.000000
vt 0.125000 0.750000
s 0
f 5/5 3/3 1/1
f 3/3 8/13 4/4
f 7/11 6/8 8/12
f 2/2 8/14 6/7
f 1/1 4/4 2/2
f 5/6 2/2 6/8
f 5/5 7/9 3/3
f 3/3 7/10 8/13
f 7/11 5/6 6/8
f 2/2 4/4 8/14
f 1/1 3/3 4/4
f 5/6 1/1 2/2
"#,
        )
        .unwrap();
        assert_eq!(obj_model.geometric_vertices.len(), 8);
        assert_eq!(obj_model.texture_vertices.len(), 14);
        assert_eq!(obj_model.faces.len(), 12);
    }
}
