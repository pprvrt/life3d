mod model;
mod universe;

#[macro_use]
extern crate glium;

use model::{Model, Vertex};
use std::f32::consts::PI;
use universe::Universe;

const WIDTH: u32 = 80;
const HEIGHT: u32 = 40;
const CYCLE: u32 = 10;

implement_vertex!(Vertex, position, normal, color);

fn main() {
    use glium::{glutin, Surface};

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Conway's game of life");
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    let mut t: f32 = 0.0;

    let mut universe = Universe::new(WIDTH, HEIGHT);
    universe.rand();

    //let cube = Model::cube();
    let cube = Model::from_obj("./resources/cube.obj");

    let vertex_buffer = glium::VertexBuffer::new(&display, &cube.vertices).unwrap();
    let index_buffer = glium::IndexBuffer::new(
        &display,
        glium::index::PrimitiveType::TrianglesList,
        &cube.indices,
    )
    .unwrap();

    let mut per_instance = {
        #[derive(Copy, Clone)]
        struct Attr {
            alive: f32,
            tick: f32,
        }

        implement_vertex!(Attr, alive, tick);

        let data = (0..WIDTH * HEIGHT)
            .map(|_| Attr {
                alive: 1.0,
                tick: 0.0,
            })
            .collect::<Vec<_>>();

        glium::vertex::VertexBuffer::dynamic(&display, &data).unwrap()
    };

    let vertex_shader_src = r#"
    #version 150
    
    in vec3 position;
    in vec3 normal;
    in float alive;
    in float tick;
    
    out vec3 vnormal;
    
    uniform mat4 perspective;
    uniform mat4 matrix;
    uniform float scaling;
    uniform uint width;

    vec3 computed = vec3(float(mod(gl_InstanceID,80)) - 80.0/2.0, float(gl_InstanceID/80) - 40/2.0, 0);
    
    float wobble = alive*sin(tick) + (1.0-alive)*cos(tick);

    void main() {
        vnormal = transpose(inverse(mat3(matrix))) * normal;
        vec4 new_position = matrix * vec4(position*wobble, scaling);
        gl_Position = perspective * vec4(computed.x + new_position.x, computed.y + new_position.y, computed.z + new_position.z, new_position.w);
    }
    "#;

    let fragment_shader_src = r#"
    #version 150
    
    in vec3 vnormal;
    out vec4 color;
    
    uniform vec3 light;
    
    void main() {
        float brightness = dot(normalize(vnormal), normalize(light));
        vec3 dark_color = vec3(0.5, 0.5, 0.5);
        vec3 light_color = vec3(1.0, 1.0, 1.0);
        
        color = vec4(mix(dark_color, light_color, brightness), 1.0);
    }
    "#;

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        blend: glium::draw_parameters::Blend::alpha_blending(),
        ..Default::default()
    };

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    let mut frame = 0;
    let mut start = std::time::Instant::now();

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            glutin::event::Event::WindowEvent {
                event: glutin::event::WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = glutin::event_loop::ControlFlow::Exit;
                return;
            }
            glutin::event::Event::NewEvents(cause) => match cause {
                glutin::event::StartCause::ResumeTimeReached { .. } => (),
                glutin::event::StartCause::Init => (),
                _ => return,
            }
            _ => return,
        }

        frame += 1;

        let next_frame_time =
            std::time::Instant::now() + std::time::Duration::from_nanos(16_666_667);
        *control_flow = glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);


        if frame % CYCLE == 0 { universe.step(); println!("fps: {}", frame as f32/start.elapsed().as_secs() as f32); }
        {
            let mut mapping = per_instance.map();
            for (id, attr) in (0..WIDTH * HEIGHT).zip(mapping.iter_mut()) {
                if frame % CYCLE == 0 {
                    attr.alive = match universe.is_alive(id as usize) {
                        true => 1.0,
                        false => 0.0,
                    };
                    if universe.has_changed(id as usize) {
                        attr.tick = 0.0
                    };
                }

                if universe.has_changed(id as usize) && attr.tick < PI/2.0 {
                    attr.tick += PI / (CYCLE as f32 * 1.5);
                }
                if attr.tick > PI/2.0 {
                    attr.tick = PI/2.0;
                }
            }
        }

        t = (t + PI / 50.0) % (PI * 2.0);

        let mut target = display.draw();
        target.clear_color_and_depth((0.0, 0.0, 0.4, 1.0), 1.0);

        let light = [-1.0, 0.4, -0.9f32];

        let perspective = {
            let (width, height) = target.get_dimensions();
            let aspect_ratio = height as f32 / width as f32;

            let fov: f32 = PI / 3.0;
            let zfar = 50.0;
            let znear = 0.1;

            let f = 1.0 / (fov / 2.0).tan();

            [
                [f * aspect_ratio, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
                [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
            ]
        };

        target.clear_color_and_depth((0.0, 0.0, 0.4, 0.5), 1.0);

        /* https://en.wikipedia.org/wiki/Rotation_matrix
         * R = Rz*Ry*Rx
         */
        let matrix = [
            [t.cos() * t.cos(), t.cos() * t.sin(), -t.sin(), 0.0],
            [
                t.sin() * t.sin() * t.cos() - t.cos() * t.sin(),
                t.sin() * t.sin() * t.sin() + t.cos() * t.cos(),
                t.sin() * t.cos(),
                0.0,
            ],
            [
                t.cos() * t.sin() * t.cos() + t.sin() * t.sin(),
                t.cos() * t.sin() * t.sin() - t.sin() * t.cos(),
                t.cos() * t.cos(),
                0.0,
            ],
            [0.0, 0.0, 30.0, 1.0f32],
        ];

        target
            .draw(
                (&vertex_buffer, per_instance.per_instance().unwrap()),
                &index_buffer,
                &program,
                &uniform! { scaling: cube.scaling,
                    matrix: matrix,
                    perspective: perspective,
                    light: light,
                    height: HEIGHT,
                    width: WIDTH },
                &params,
            )
            .unwrap();
        target.finish().unwrap();
    });
}
