extern crate tobj;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub color: [f32; 3],
}

#[derive(Clone)]
pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub scaling: f32,
}

impl Model {
    pub fn from_obj(obj_file: &str) -> Model {
        let mut min_pos = [f32::INFINITY; 3];
        let mut max_pos = [f32::NEG_INFINITY; 3];

        let (models, _) = tobj::load_obj(
            obj_file,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..tobj::LoadOptions::default()
            },
        )
        .expect("Failed to OBJ load file");

        if models.len() > 1 {
            panic!("Cannot handle more than one model per obj.")
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for model in models {
            let mesh = &model.mesh;
            let mut count = 0;
            for idx in &mesh.indices {
                let i = *idx as usize;
                let position = [
                    mesh.positions[3 * i],
                    mesh.positions[3 * i + 1],
                    mesh.positions[3 * i + 2],
                ];

                indices.extend([count, count + 1, count + 2]);
                count += 3;
                let normal = if !mesh.normals.is_empty() {
                    [
                        mesh.normals[3 * i],
                        mesh.normals[3 * i + 1],
                        mesh.normals[3 * i + 2],
                    ]
                } else {
                    [0.0, 0.0, 0.0]
                };

                vertices.push(Vertex {
                    position,
                    normal,
                    color: position,
                });

                for i in 0..3 {
                    min_pos[i] = f32::min(min_pos[i], position[i]);
                    max_pos[i] = f32::max(max_pos[i], position[i]);
                }
            }
        }
        let diagonal_len = 3.0;
        let current_len = f32::powf(max_pos[0] - min_pos[0], 2.0)
            + f32::powf(max_pos[1] - min_pos[1], 2.0)
            + f32::powf(max_pos[2] - min_pos[2], 2.0);
        let scaling = f32::sqrt(diagonal_len / current_len);

        for vertex in vertices.iter_mut() {
            for i in 0..3 {
                vertex.position[i] *= scaling;
            }
        }

        Model {
            vertices,
            indices,
            scaling,
        }
    }
}
