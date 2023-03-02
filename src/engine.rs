use nalgebra::Perspective3;

use crate::support;
use crate::universe::Universe;
use glium::Surface;
use std::f32::consts::PI;

const SHORTEST_LIFECYCLE: u32 = 2;
const LONGEST_LIFECYCLE: u32 = 60;

pub struct Mouse {
    x: u16,
    y: u16,
}

impl Mouse {
    pub fn x(&self) -> u16 {
        self.x
    }
    pub fn y(&self) -> u16 {
        self.y
    }
}

#[derive(Debug, PartialEq)]
pub enum EngineState {
    Running,
    Stopped,
}

#[derive(Copy, Clone, PartialEq)]
pub enum EngineDrawState {
    Drawing,
    None,
}

#[derive(Copy, Clone)]
pub enum EngineEvent {
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
    event: EngineEvent,
    mouse: Mouse,
    frame: u32,
    lifecycle: u32,
    t: f32,
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
            event: EngineEvent::None,
            mouse: Mouse { x: 0, y: 0 },
            frame: 0,
            lifecycle: u32::min(u32::max(lifecycle, SHORTEST_LIFECYCLE), LONGEST_LIFECYCLE),
            t: 0.0,
        }
    }

    pub fn lifecycle(&self) -> u32 {
        self.lifecycle
    }

    pub fn t(&self) -> f32 {
        self.t
    }

    pub fn set_mouse(&mut self, mx: u16, my: u16) {
        self.mouse.x = mx;
        self.mouse.y = my;
    }

    pub fn is_drawing(&self) -> bool {
        self.draw.state == EngineDrawState::Drawing
    }

    pub fn start_drawing(&mut self) {
        self.draw.state = EngineDrawState::Drawing
    }

    pub fn stop_drawing(&mut self) {
        self.draw.cx = -1;
        self.draw.cy = -1;
        self.draw.state = EngineDrawState::None;
    }

    pub fn just_drawn(&self, cx: i32, cy: i32) -> bool {
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

    pub fn poll(&mut self) -> EngineEvent {
        let event = self.event;
        self.event = EngineEvent::None;
        event
    }

    pub fn trigger(&mut self, event: EngineEvent) {
        self.event = event;
    }

    pub fn next_frame(&mut self) {
        self.frame = (self.frame + 1) % self.lifecycle;
    }

    pub fn change_lifecycle(&mut self, delta: i32) {
        let lifecycle = i32::max(self.lifecycle as i32 + delta, SHORTEST_LIFECYCLE as i32) as u32;
        self.lifecycle = u32::min(lifecycle, LONGEST_LIFECYCLE);
        self.frame = u32::min(self.frame, self.lifecycle - 1);
    }

    pub fn step(
        &mut self,
        universe: &mut Universe,
        target: &mut glium::Frame,
        camera: &mut support::Camera,
        projection_matrix: &Perspective3<f32>,
        per_instance: &mut glium::VertexBuffer<support::CellAttr>,
    ) {
        self.t = (self.t + PI / 45.0) % (PI * 2.0);

        if self.is_drawing() {
            /* Project the mouse 2D position into the 3D world */
            if let Some([cx, cy]) =
                support::mouse_projection(target, self.mouse(), camera, projection_matrix, universe)
            {
                if !self.just_drawn(cx as i32, cy as i32) {
                    universe.toggle(cx, cy);
                    self.draw(cx as i32, cy as i32);
                }
            }
        }

        camera.step();

        if self.is_running() {
            target.clear_color_and_depth((0.0, 0.0, 0.2, 0.8), 1.0);
        } else {
            target.clear_color_and_depth((0.4, 0.0, 0.0, 0.8), 1.0);
        }

        support::update_dynamic_attributes(per_instance, universe, self);

        /* Handle engine events instantly */
        match self.poll() {
            EngineEvent::Randomize => {
                universe.rand();
                self.reset();
            }
            EngineEvent::Clear => {
                universe.clear();
                self.reset();
            }
            _ => (),
        }

        /* If the engine is running, progress. If not, wait until
        the end of a generation to pause */
        if self.is_running() || !self.is_last_frame() {
            self.next_frame();
        }

        /* It's a new dawn, it's a new day, it's new a life */
        if self.is_first_frame() {
            universe.step();
        }
    }

    pub fn reset(&mut self) {
        self.frame = 0;
    }

    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }
}
