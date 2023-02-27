#![allow(dead_code)]

mod engine;
mod model;
mod support;
mod universe;

#[macro_use]
extern crate glium;

use engine::{Engine, EngineEvent};
use model::{Model, Vertex};
use std::f32::consts::PI;
use support::{model_matrix, mouse_projection, perspective_matrix, Camera, CellAttr};
use universe::Universe;

// Width and height of Conway's universe
const WIDTH: usize = 60;
const HEIGHT: usize = 60;
// Number of cycles before a new generation
const LIFECYCLE: u32 = 24;

implement_vertex!(Vertex, position, normal, color);

fn main() {
    use glium::{glutin, Surface};
    use glutin::event;

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Conway's game of life");
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // Create engine and universe
    let mut engine = Engine::new(LIFECYCLE);
    let mut universe = Universe::new(WIDTH, HEIGHT);
    universe.rand();

    // Start with time at 0
    let mut t: f32 = 0.0;

    // Load cube model from OBJ
    let cube = Model::from_obj("./resources/cube.obj");

    let vertex_buffer = glium::VertexBuffer::new(&display, &cube.vertices).unwrap();

    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &cube.indices,
    )
    .unwrap();

    let mut per_instance = {
        implement_vertex!(CellAttr, alive, tick);

        let data = (0..universe.size())
            .map(|_| CellAttr {
                alive: 1.0,
                tick: 1.0,
            })
            .collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        smooth: Some(glium::draw_parameters::Smooth::Fastest),
        blend: glium::draw_parameters::Blend::alpha_blending(),
        backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        ..Default::default()
    };

    let program = glium::Program::from_source(
        &display,
        support::vertex_shader(),
        support::fragment_shader(),
        None,
    )
    .unwrap();

    /* Light source */
    let light = [-1.0, 0.4, -0.9f32];

    /* Camera */
    let camera = Camera::new([0.0, 0.0, -25.0], [0.0, 8.0, 1.0], [0.0, 1.0, 0.0]);

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                event::WindowEvent::KeyboardInput { input, .. } => match input {
                    event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::R),
                        state: event::ElementState::Pressed,
                        ..
                    } => {
                        engine.trigger(EngineEvent::Randomize);
                        return;
                    }
                    event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::Delete),
                        state: event::ElementState::Pressed,
                        ..
                    } => {
                        engine.trigger(EngineEvent::Clear);
                        return;
                    }
                    event::KeyboardInput {
                        virtual_keycode: Some(event::VirtualKeyCode::Space),
                        state: event::ElementState::Pressed,
                        ..
                    } => {
                        engine.startstop();
                        return;
                    }
                    _ => return,
                },
                event::WindowEvent::CursorMoved { position, .. } => {
                    engine.set_mouse(position.x as u16, position.y as u16);
                    return;
                }
                event::WindowEvent::MouseInput {
                    button: event::MouseButton::Left,
                    state,
                    ..
                } => {
                    match state {
                        event::ElementState::Pressed => engine.start_drawing(),
                        event::ElementState::Released => engine.stop_drawing(),
                    };
                    return;
                }
                _ => return,
            },
            event::Event::NewEvents(cause) => match cause {
                event::StartCause::ResumeTimeReached { .. } => (),
                event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }
        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

        let mut target = display.draw();

        let model_matrix = model_matrix(t, t, t);
        let projection_matrix = perspective_matrix(&target);

        t = (t + PI / 45.0) % (PI * 2.0);

        if engine.is_drawing() {
            /* Project the mouse 2D position into the 3D world */
            if let Some([cx, cy]) = mouse_projection(
                &target,
                engine.mouse(),
                &camera,
                &projection_matrix,
                &universe,
            ) {
                if !engine.just_drawn(cx as i32, cy as i32) {
                    universe.toggle(cx, cy);
                    engine.draw(cx as i32, cy as i32);
                }
            }
        }

        if engine.is_running() {
            target.clear_color_and_depth((0.0, 0.0, 0.2, 0.8), 1.0);
        } else {
            target.clear_color_and_depth((0.4, 0.0, 0.0, 0.8), 1.0);
        }

        support::vertex_dynamic_attributes(&mut per_instance, &universe, &engine);

        target
            .draw(
                (&vertex_buffer, per_instance.per_instance().unwrap()),
                &index_buffer,
                &program,
                &uniform! {
                u_model: *model_matrix.to_homogeneous().as_ref(),
                u_view: *camera.view_matrix().to_homogeneous().as_ref(),
                u_perspective: *projection_matrix.to_homogeneous().as_ref(),
                u_light: light,
                u_height: universe.height() as i32,
                u_width: universe.width() as i32},
                &params,
            )
            .unwrap();
        target.finish().unwrap();

        /* Handle engine events instantly */
        match engine.poll() {
            EngineEvent::Randomize => {
                universe.rand();
                engine.reset();
            }
            EngineEvent::Clear => {
                universe.clear();
                engine.reset();
            }
            _ => (),
        }

        /* If the engine is running, progress. If not, wait until
        the end of a generation to pause */
        if engine.is_running() || !engine.is_last_frame() {
            engine.step();
        }

        /* It's a new dawn, it's a new day, it's new a life */
        if engine.is_first_frame() {
            universe.step();
        }
    });
}
