use nalgebra as na;
use crate::engine::{Mouse,Engine};
use crate::universe::Universe;
use std::f32::consts::PI;
use glium::VertexBuffer;


#[derive(Copy, Clone)]
pub struct CellAttr {
    pub alive: f32,
    pub tick: f32,
}

pub struct Camera {
    position: [f32; 3],
    destination: [f32; 3],
    direction: [f32; 3],
    velocity: [f32; 3],
    up: [f32; 3],
    view: na::Isometry3<f32>,
    dt: f32
}

impl Camera {
    fn build_matrix(
        position: &[f32; 3],
        direction: &[f32; 3],
        up: &[f32; 3],
    ) -> na::Isometry3<f32> {
        let eye = na::Point3::from_slice(position);
        let target = na::Point3::from_slice(direction);
        let up = na::Vector3::from_row_slice(up);
        na::Isometry3::look_at_rh(&eye, &target, &up)
    }

    pub fn shift(&mut self, z: f32) {
        let mut dest_z = self.position[2] + z;
        dest_z = f32::max(10.0, dest_z);
        dest_z = f32::min(30.0, dest_z);
        self.destination = [self.position[0], self.position[1], dest_z];
        self.dt = 0.0;
    }

    pub fn step(&mut self) {
        let freq = 0.05;

        self.dt += 1.0f32;
        let exp_term = f32::exp(-freq * self.dt);
        let time_exp = self.dt * exp_term;
        let time_exp_freq = time_exp * freq;

        for i in 0..3 {
            self.position[i] = (self.position[i] - self.destination[i]) * (time_exp_freq + exp_term) +
                self.velocity[i] * time_exp + self.destination[i];
            self.velocity[i] = (self.position[i] - self.destination[i]) * (-freq * time_exp_freq) +
                self.velocity[i] * (-time_exp_freq + exp_term);
        }
        self.view = Camera::build_matrix(&self.position, &self.direction, &self.up);
    }
    pub fn new(position: [f32; 3], direction: [f32; 3], up: [f32; 3]) -> Self {
        Camera {
            position,
            direction,
            up,
            view: Camera::build_matrix(&position, &direction, &up),
            velocity: [0.0, 0.0, 0.0],
            destination: position,
            dt: 0.0
        }
    }

    pub fn view_matrix(&self) -> &na::Isometry3<f32> {
        &self.view
    }
}

pub fn mouse_projection(
    target: &impl glium::Surface,
    mouse: &Mouse,
    camera: &Camera,
    perspective: &na::Perspective3<f32>,
    universe: &Universe,
) -> Option<[usize; 2]> {

    let (width, height) = target.get_dimensions();
    let ray_clip = na::Vector4::new(
        2.0 * mouse.x() as f32 / width as f32 - 1.0,
        1.0 - 2.0 * mouse.y() as f32 / height as f32,
        -1.0,
        1.0
    );

    let (u_width, u_height) = universe.dimensions();

    let mut ray_eye = perspective.inverse() * ray_clip;
    (ray_eye.z, ray_eye.w) = (-1.0, 0.0);

    let mut ray_world = (camera.view.inverse().to_homogeneous() * ray_eye).xyz();
    ray_world.normalize_mut();

    let t = -camera.position[2] / ray_world[2];
    let x = camera.position[0] + ray_world[0] * t + u_width as f32 / 2.0 + 0.5;
    let y = camera.position[1] + ray_world[1] * t + u_height as f32 / 2.0 + 0.5;

    if x >= 0.0 && y >= 0.0 && x < u_width as f32 && y < u_height as f32 {
        Some([x as usize, y as usize])
    } else {
        None
    }
}

pub fn perspective_matrix(target: &impl glium::Surface) -> na::Perspective3<f32> {
    let (width, height) = target.get_dimensions();
    na::Perspective3::new(width as f32 / height as f32, PI / 3.0, 0.1, 1024.0)
}

pub fn model_matrix(roll: f32, pitch: f32, yaw: f32) -> na::Rotation3<f32> {
    na::Rotation3::from_euler_angles(roll, pitch, yaw)
}

pub fn init_dynamic_attributes(display: &glium::backend::glutin::Display, universe: &Universe) -> VertexBuffer<CellAttr>
{
    let data = (0..universe.size())
    .map(|_| CellAttr {
        alive: 1.0,
        tick: 1.0,
    })
    .collect::<Vec<_>>();
    glium::vertex::VertexBuffer::dynamic(display, &data).unwrap()
}

pub fn update_dynamic_attributes(per_instance: &mut VertexBuffer<CellAttr>, universe: &Universe, engine: &Engine)
{
    let mut mapping = per_instance.map_write();
    for id in 0..universe.size() {
        mapping.set(id, CellAttr {
            alive: match universe.is_alive(id) {
                true => 1.0,
                false => 0.0,
            },
            tick: if universe.has_changed(id) {
                f32::min(1.0, engine.frame() as f32 / (engine.lifecycle() - 1) as f32)
            } else {
                /* We might have reset the universe in-between generations, we cannot
                 * assume that unchanged cells were fully alive or dead */
                1.0
            }
        });        
    }
}

pub fn vertex_shader() -> &'static str {
    include_str!("../shaders/vertex.glsl")
}

pub fn fragment_shader() -> &'static str {
    include_str!("../shaders/fragment.glsl")
}
