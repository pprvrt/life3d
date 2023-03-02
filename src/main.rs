#![allow(dead_code)]

mod engine;
mod model;
mod support;
mod universe;

use engine::{Engine, EngineEvent};
use model::{Model, Vertex};
use support::{Camera, CellAttr};
use universe::Universe;

use glium::{implement_vertex, uniform};

// Width and height of Conway's universe
const WIDTH: usize = 60;
const HEIGHT: usize = 60;
// Number of cycles before a new generation
const LIFECYCLE: u32 = 24;
const WAITFRAME: u64 = 16_666_667;

implement_vertex!(Vertex, position, normal, color);
implement_vertex!(CellAttr, alive, tick);

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

    // Load cube model from OBJ
    let cube = Model::from_obj("./resources/cube.obj");

    let vertex_buffer = glium::VertexBuffer::new(&display, &cube.vertices).unwrap();

    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &cube.indices,
    )
    .unwrap();

    let mut per_instance = support::init_dynamic_attributes(&display, &universe);

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        smooth: Some(glium::draw_parameters::Smooth::Fastest),
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
    let light = [0.0, 0.0, 10f32];

    /* Camera */
    let mut camera = Camera::new([0.0, 0.0, 25.0], [0.0, 8.0, -1.0], [0.0, 1.0, 0.0]);
    let mut now = std::time::Instant::now();
    let mut accumulator: u128 = 0;

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                event::WindowEvent::KeyboardInput { input, .. } => match input {
                    event::KeyboardInput {
                        virtual_keycode,
                        state: event::ElementState::Pressed,
                        ..
                    } => match virtual_keycode {
                        Some(event::VirtualKeyCode::R) => {
                            engine.trigger(EngineEvent::Randomize);
                            return
                        }
                        Some(event::VirtualKeyCode::Delete) => {
                            engine.trigger(EngineEvent::Clear);
                            return
                        }
                        Some(event::VirtualKeyCode::Space) => {
                            engine.startstop();
                            return
                        }
                        Some(event::VirtualKeyCode::Left) => {
                            engine.change_lifecycle(2);
                            return
                        }
                        Some(event::VirtualKeyCode::Right) => {
                            engine.change_lifecycle(-2);
                            return
                        }
                        _ => return,
                    },
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
                event::WindowEvent::MouseWheel { delta, .. } => match delta {
                    event::MouseScrollDelta::LineDelta(_, delta) => {
                        camera.shift(-delta * 20.0);
                        return;
                    }
                    event::MouseScrollDelta::PixelDelta(pos) => {
                        camera.shift(10.0 * pos.y as f32);
                        return;
                    }
                },
                _ => return,
            },
            event::Event::NewEvents(cause) => match cause {
                event::StartCause::ResumeTimeReached { .. } => (),
                event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        let mut target = display.draw();
        let model_matrix = support::model_matrix(engine.t(), engine.t(), engine.t());
        let projection_matrix = support::perspective_matrix(&target);

        accumulator += now.elapsed().as_nanos();
        now = std::time::Instant::now();
        while accumulator >= WAITFRAME as u128 {
            engine.step(
                &mut universe,
                &mut target,
                &mut camera,
                &projection_matrix,
                &mut per_instance,
            );
            accumulator -= WAITFRAME as u128;
        }
        let next_frame_time = now + std::time::Duration::from_nanos(WAITFRAME - accumulator as u64);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

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
                u_width: universe.width() as i32,
                u_height: universe.height() as i32},
                &params,
            )
            .unwrap();
        target.finish().unwrap();
    });
}
