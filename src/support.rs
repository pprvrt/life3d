extern crate nalgebra as na;
extern crate glium;

use crate::Universe;
use std::f32::consts::PI;

pub struct Mouse {
    x: u16,
    y: u16,
}

#[derive(Debug, PartialEq)]
pub enum EngineState {
    Running,
    Stopped,
}

#[derive(Copy, Clone)]
pub enum EngineDrawState {
    Drawing,
    None,
}

#[derive(Copy, Clone)]
pub enum EngineEvents {
    Randomize,
    Clear,
    None,
}

struct Draw {
    cx: i32,
    cy: i32,
    state: EngineDrawState,
}

pub struct Engine {
    state: EngineState,
    draw: Draw,
    event: EngineEvents,
    mouse: Mouse,
    frame: u32,
    lifecycle: u32
}

pub struct Camera {
    position: [f32; 3],
    direction: [f32; 3],
    up: [f32; 3],
    matrix: na::Isometry3<f32>,
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

    pub fn new(position: [f32; 3], direction: [f32; 3], up: [f32; 3]) -> Self {
        Camera {
            position,
            direction,
            up,
            matrix: Camera::build_matrix(&position, &direction, &up),
        }
    }

    pub fn view_matrix(&self) -> &na::Isometry3<f32> {
        &self.matrix
    }

}

impl Engine {
    pub fn new(lifecycle: u32) -> Self {
        Engine {
            state: EngineState::Running,
            draw: Draw {
                cx: -1,
                cy: -1,
                state: EngineDrawState::None,
            },
            event: EngineEvents::None,
            mouse: Mouse { x: 0, y: 0 },
            frame: 0,
            lifecycle
        }
    }

    pub fn set_mouse(&mut self, mx: u16, my: u16) {
        self.mouse.x = mx;
        self.mouse.y = my;
    }

    pub fn draw_state(&self) -> EngineDrawState {
        return self.draw.state;
    }

    pub fn start_drawing(&mut self) {
        self.draw.state = EngineDrawState::Drawing
    }

    pub fn stop_drawing(&mut self) {
        self.draw.cx = -1;
        self.draw.cy = -1;
        self.draw.state = EngineDrawState::None;
    }

    pub fn has_drawn(&self, cx: i32, cy: i32) -> bool {
        self.draw.cx == cx && self.draw.cy == cy
    }

    pub fn draw(&mut self, cx: i32, cy: i32) {
        self.draw.cx = cx;
        self.draw.cy = cy;
    }

    pub fn startstop(&mut self) {
        self.state = match self.state {
            EngineState::Running => EngineState::Stopped,
            EngineState::Stopped => EngineState::Running,
        };
    }

    pub fn is_running(&self) -> bool {
        self.state == EngineState::Running
    }

    pub fn is_last_frame(&self) -> bool {
        self.frame == self.lifecycle - 1
    }

    pub fn is_first_frame(&self) -> bool {
        self.frame == 0
    }

    pub fn frame(&self) -> u32 {
        self.frame
    }

    pub fn poll(&mut self) -> EngineEvents {
        let event = self.event;
        self.event = EngineEvents::None;
        event
    }

    pub fn trigger(&mut self, event: EngineEvents) {
        self.event = event;
    }

    pub fn step(&mut self) {
        self.frame = (self.frame + 1) % self.lifecycle;
    }

    pub fn reset(&mut self) {
        self.frame = 0;
    }

    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }
}

pub fn mouse_projection(
    width: u32,
    height: u32,
    mouse: &Mouse,
    camera: &Camera,
    perspective: &na::Perspective3<f32>,
    universe: &Universe
) -> Option<[usize; 2]> {
    let ray = na::Vector3::new(
        2.0 * mouse.x as f32 / width as f32 - 1.0,
        1.0 - 2.0 * mouse.y as f32 / height as f32,
        0.0,
    )
    .to_homogeneous();

    let (u_width, u_height) = universe.dimensions();
    
    let mut ray_eye = perspective.inverse() * ray;
    (ray_eye.z, ray_eye.w) = (-1.0, 0.0);
    
    let mut ray_world =
    (camera.matrix.inverse().to_homogeneous() * ray_eye).xyz();
    ray_world.normalize_mut();
    
    let t = -camera.position[2] / ray_world[2];
    let x = camera.position[0] + ray_world[0] * t + 0.5 + u_width as f32 / 2.0;
    let y = camera.position[1] + ray_world[1] * t + 0.5 + u_height as f32 / 2.0;
    
    if x >= 0.0
    && y >= 0.0
    && x < u_width as f32
    && y < u_height as f32
    {
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