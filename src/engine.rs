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
        self.draw.state
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
