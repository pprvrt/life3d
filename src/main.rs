mod model;
mod universe;

#[macro_use]
extern crate glium;

use model::{Model, Vertex};
use std::f32::consts::PI;
use universe::Universe;

// Width and height of Conway's universe
const WIDTH: usize = 60;
const HEIGHT: usize = 60;
// Number of cycles before a new generation
const LIFECYCLE: u32 = 40;

implement_vertex!(Vertex, position, normal, color);

fn get_perspective(target: &impl glium::Surface) -> [[f32; 4]; 4] {
    let (width, height) = target.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = PI / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f * aspect_ratio, 0.0, 0.0, 0.0],
        [0.0, f, 0.0, 0.0],
        [0.0, 0.0, (zfar + znear) / (zfar - znear), 1.0],
        [0.0, 0.0, -(2.0 * zfar * znear) / (zfar - znear), 0.0],
    ]
}

fn get_matrix(t: f32) -> [[f32; 4]; 4] {
    /* https://en.wikipedia.org/wiki/Rotation_matrix
     * R = Rz*Ry*Rx
     */
    [
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
        [0.0, 0.0, 20.0, 1.0f32],
    ]
}

fn main() {
    use glium::{glutin, Surface};
    use glutin::event;

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Conway's game of life");
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // Start with time at 0
    let mut t: f32 = 0.0;

    // Generate universe
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
                tick: 1.0,
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
    out float valive;
    out float vtick;
    
    uniform mat4 perspective;
    uniform mat4 matrix;
    uniform float scaling;
    uniform int width;
    uniform int height;

    /* https://github.com/glslify/glsl-easings/blob/master/bounce-out.glsl */
    float bounceOut(float t) {
        const float a = 4.0 / 11.0;
        const float b = 8.0 / 11.0;
        const float c = 9.0 / 10.0;
      
        const float ca = 4356.0 / 361.0;
        const float cb = 35442.0 / 1805.0;
        const float cc = 16061.0 / 1805.0;
      
        float t2 = t * t;
      
        return t < a
          ? 7.5625 * t2
          : t < b
            ? 9.075 * t2 - 9.9 * t + 3.4
            : t < c
              ? ca * t2 - cb * t + cc
              : t > 1.0
                ? 1.0
                : 10.8 * t * t - 20.52 * t + 10.72;
    }

    vec4 grid = vec4(float(mod(gl_InstanceID,width)) - float(width)/2.0, float(gl_InstanceID/width) - float(height)/2.0, 0, 0);

    float wobble = alive*bounceOut(tick*1.2) + (1.0-alive)*(1-smoothstep(0.0,0.5,tick));

    void main() {
        valive = alive;
        vtick = tick;
        /* Transform normal vector with transformation matrix */
        vnormal = transpose(inverse(mat3(matrix))) * normal;
        vec4 origin = matrix * vec4(position * wobble * scaling, 1);
        gl_Position = perspective * (grid + origin);
    }
    "#;

    let fragment_shader_src = r#"
    #version 150
    
    in float valive;
    in float vtick;
    in vec3 vnormal;
    out vec4 color;

    vec3 white = vec3(1.0, 1.0, 1.0);
    vec3 black = vec3(0.5, 0.5, 0.5);
    
    uniform vec3 light;
    
    /* Simple Gouraud shading */
    void main() {
        vec3 lightc = mix(valive*vec3(0.0, 0.6, 0.0) + (1.0-valive)*vec3(0.6, 0.0, 0.0), white, vtick);
        vec3 darkc = mix(valive*vec3(0.0, 0.3, 0.0) + (1.0-valive)*vec3(0.3, 0.0, 0.0), black, vtick);
        float brightness = dot(normalize(vnormal), normalize(light));
        color = vec4(mix(darkc, lightc, brightness), 1.0);
    }
    "#;

    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        smooth: Some(glium::draw_parameters::Smooth::Fastest),
        blend: glium::draw_parameters::Blend::alpha_blending(),
        ..Default::default()
    };

    let program =
        glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None)
            .unwrap();

    /* Frame counter */
    let mut frame = 0;

    /* Light source */
    let light = [-1.0, 0.4, -0.9f32];

    let mut randomize = false;

    let start = std::time::Instant::now();
    event_loop.run(move |ev, _, control_flow| {
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                event::WindowEvent::KeyboardInput {
                    input:
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Space),
                            state: event::ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    randomize = true;
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

        {
            let mut mapping = per_instance.map();
            for (id, attr) in (0..WIDTH * HEIGHT).zip(mapping.iter_mut()) {
                if frame % LIFECYCLE == 0 {
                    attr.alive = match universe.is_alive(id) {
                        true => 1.0,
                        false => 0.0,
                    };
                    if universe.has_changed(id) {
                        attr.tick = 0.0
                    };
                }

                if universe.has_changed(id) {
                    attr.tick += 1.0 / LIFECYCLE as f32;
                }
            }
        }

        t = (t + PI / 45.0) % (PI * 2.0);

        let mut target = display.draw();

        target.clear_color_and_depth((0.0, 0.0, 0.4, 0.8), 1.0);

        target
            .draw(
                (&vertex_buffer, per_instance.per_instance().unwrap()),
                &index_buffer,
                &program,
                &uniform! { scaling: cube.scaling,
                matrix: get_matrix(t),
                perspective: get_perspective(&target),
                light: light,
                height: HEIGHT as i32,
                width: WIDTH as i32},
                &params,
            )
            .unwrap();
        target.finish().unwrap();
        frame += 1;
        if frame % LIFECYCLE == 0 {
            if randomize {
                universe.rand();
                randomize = false;
            } 
            else {
                universe.step();
                println!("fps: {}", frame as f32/start.elapsed().as_secs() as f32)
            }
        }
    });
}
