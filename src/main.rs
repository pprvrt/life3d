#![allow(dead_code)]

mod model;
mod universe;

#[macro_use]
extern crate glium;
extern crate nalgebra as na;

use model::{Model, Vertex};
use std::f32::consts::PI;
use universe::Universe;

// Width and height of Conway's universe
const WIDTH: usize = 60;
const HEIGHT: usize = 60;
// Number of cycles before a new generation
const LIFECYCLE: u32 = 40;

implement_vertex!(Vertex, position, normal, color);

struct Mouse {
    x: u16,
    y: u16,
}

struct Camera {
    position: [f32; 3],
    direction: [f32; 3],
    up: [f32; 3],
    matrix: na::Isometry3<f32>,
}

impl Camera {
    fn new(position: [f32; 3], direction: [f32; 3], up: [f32; 3]) -> Self {
        Camera {
            position,
            direction,
            up,
            matrix: view_matrix(&position, &direction, &up),
        }
    }
}

fn mouse_projection(
    width: u32,
    height: u32,
    mouse: &Mouse,
    camera: &Camera,
    perspective: &na::Perspective3<f32>,
) -> [usize; 2] {
    let ray = na::Vector3::new(
        2.0 * mouse.x as f32 / width as f32 - 1.0,
        1.0 - 2.0 * mouse.y as f32 / height as f32,
        0.0,
    )
    .to_homogeneous();

    let mut ray_eye = perspective.inverse() * ray;
    (ray_eye.z, ray_eye.w) = (-1.0, 0.0);

    let mut ray_world =
        (camera.matrix.inverse().to_homogeneous() * ray_eye).xyz();
    ray_world.normalize_mut();

    let t = -camera.position[2] / ray_world[2];
    let x = camera.position[0] + ray_world[0] * t + 0.5 + WIDTH as f32 / 2.0;
    let y = camera.position[1] + ray_world[1] * t + 0.5 + HEIGHT as f32 / 2.0;

    [x as usize, y as usize]
}

fn perspective_matrix(target: &impl glium::Surface) -> na::Perspective3<f32> {
    let (width, height) = target.get_dimensions();
    na::Perspective3::new(width as f32 / height as f32, PI / 3.0, 0.1, 1024.0)
}

fn model_matrix(roll: f32, pitch: f32, yaw: f32) -> na::Rotation3<f32> {
    na::Rotation3::from_euler_angles(roll, pitch, yaw)
}

fn view_matrix(
    position: &[f32; 3],
    direction: &[f32; 3],
    up: &[f32; 3],
) -> na::Isometry3<f32> {
    let eye = na::Point3::from_slice(position);
    let target = na::Point3::from_slice(direction);
    let up = na::Vector3::from_row_slice(up);
    na::Isometry3::look_at_rh(&eye, &target, &up)
}

fn main() {
    use glium::{glutin, Surface};
    use glutin::event;

    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new()
        .with_title("Conway's game of life");
    let cb = glutin::ContextBuilder::new().with_depth_buffer(24);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // Start with time at 0
    let mut t: f32 = 0.0;

    // Generate universe
    let mut universe = Universe::new(WIDTH, HEIGHT);
    universe.rand();

    // Load cube model from OBJ
    let cube = Model::from_obj("./resources/cube.obj");

    let vertex_buffer =
        glium::VertexBuffer::new(&display, &cube.vertices).unwrap();

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
    
    out vec3 v_normal;
    out vec3 v_position;
    out float v_alive;
    out float v_tick;
    
    uniform mat4 u_view;
    uniform mat4 u_perspective;
    uniform mat4 u_model;
    uniform int u_width;
    uniform int u_height;

    vec4 instance = vec4(float(mod(gl_InstanceID, u_width)) - float(u_width)/2.0,
        float(gl_InstanceID/u_width) - float(u_height)/2.0, 0, 0);

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

    float wobble = alive*bounceOut(tick*1.2) + (1.0-alive)*(1-smoothstep(0.0,0.5,tick));

    void main() {
        v_alive = alive;
        v_tick = tick;

        /* Transform normal vector with model transformation matrix */
        v_normal = transpose(inverse(mat3(u_model))) * normal;
        /* Transform the instance according to the wobble birth&death effect */
        vec4 origin = u_model * vec4(position * wobble, 1);
        /* Move the instance on the grid, apply camera transformation and perspective transformation */
        gl_Position = u_perspective * u_view * (instance + origin);
        v_position = gl_Position.xyz / gl_Position.w;
    }
    "#;

    let fragment_shader_src = r#"
    #version 150
    
    in float v_alive;
    in float v_tick;
    in vec3 v_normal;
    in vec3 v_position;

    out vec4 color;

    uniform vec3 u_light;

    const vec3 ambient = vec3(0.3, 0.3, 0.3);
    const vec3 diffuse = vec3(0.6, 0.6, 0.6);

    vec3 ambient_color = v_alive*mix(vec3(0.0, 0.2, 0.0), ambient, v_tick) 
        + (1.0-v_alive)*mix(ambient, vec3(0.2, 0.0, 0.0), v_tick*2.5);
    vec3 diffuse_color = v_alive*mix(vec3(0.0, 0.6, 0.0), diffuse, v_tick) 
        + (1.0-v_alive)*mix(diffuse, vec3(0.6, 0.0, 0.0), v_tick*2.5);
    vec3 specular_color = vec3(1.0, 1.0, 1.0);

    void main() {
        float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);
        vec3 camera_dir = normalize(-v_position);
        vec3 half_direction = normalize(normalize(u_light)+camera_dir);
        float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 80.0);

        color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
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

    let program = glium::Program::from_source(
        &display,
        vertex_shader_src,
        fragment_shader_src,
        None,
    )
    .unwrap();

    /* Frame counter */
    let mut frame = 0;

    /* Light source */
    let light = [-1.0, 0.4, -0.9f32];

    /* Camera */
    let camera =
        Camera::new([0.0, 0.0, -25.0], [0.0, 0.6, 1.0], [0.0, 1.0, 0.0]);

    let mut clear = false;
    let mut randomize = false;
    let mut stop = false;
    let mut toggle = false;

    let mut mouse = Mouse { x: 0, y: 0 };

    event_loop.run(move |ev, _, control_flow| {
        match ev {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                    return;
                }
                event::WindowEvent::KeyboardInput { input, .. } => {
                    match input {
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::Space),
                            state: event::ElementState::Pressed,
                            ..
                        } => {
                            randomize = true;
                        }
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::X),
                            state: event::ElementState::Pressed,
                            ..
                        } => {
                            clear = true;
                        }
                        event::KeyboardInput {
                            virtual_keycode: Some(event::VirtualKeyCode::S),
                            state: event::ElementState::Pressed,
                            ..
                        } => {
                            stop = !stop;
                        }
                        _ => (),
                    }
                }
                event::WindowEvent::CursorMoved { position, .. } => {
                    (mouse.x, mouse.y) = (position.x as u16, position.y as u16);
                }
                event::WindowEvent::MouseInput {
                    button: event::MouseButton::Left,
                    state: event::ElementState::Pressed,
                    ..
                } => {
                    toggle = true;
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

        let mut target = display.draw();
        let (target_x, target_y) = target.get_dimensions();

        t = (t + PI / 45.0) % (PI * 2.0);

        let model_matrix = model_matrix(t, t, t);
        let projection_matrix = perspective_matrix(&target);

        let next_frame_time = std::time::Instant::now()
            + std::time::Duration::from_nanos(16_666_667);
        *control_flow =
            glutin::event_loop::ControlFlow::WaitUntil(next_frame_time);

            if toggle {
                let [x, y] = mouse_projection(
                    target_x,
                    target_y,
                    &mouse,
                    &camera,
                    &projection_matrix,
                );
                universe.toggle(x, y);
                toggle = false;
            }

        {
            let mut mapping = per_instance.map();
            for (id, attr) in (0..WIDTH * HEIGHT).zip(mapping.iter_mut()) {
                attr.alive = match universe.is_alive(id) {
                    true => 1.0,
                    false => 0.0,
                };
                if universe.has_changed(id) {
                    attr.tick = (frame % LIFECYCLE) as f32 / LIFECYCLE as f32;
                }
            }
        }

        target.clear_color_and_depth((0.0, 0.0, 0.4, 0.8), 1.0);

        target
            .draw(
                (&vertex_buffer, per_instance.per_instance().unwrap()),
                &index_buffer,
                &program,
                &uniform! {
                u_model: *model_matrix.to_homogeneous().as_ref(),
                u_view: *camera.matrix.to_homogeneous().as_ref(),
                u_perspective: *projection_matrix.to_homogeneous().as_ref(),
                u_light: light,
                u_height: HEIGHT as i32,
                u_width: WIDTH as i32},
                &params,
            )
            .unwrap();
        target.finish().unwrap();

        if !stop || (stop && frame % LIFECYCLE < LIFECYCLE-1) {
            frame += 1;
        }
        if frame % LIFECYCLE == 0 {
            if randomize || clear {
                if randomize {
                    universe.rand();
                    randomize = false;
                }
                if clear {
                    universe.clear();
                    clear = false;
                }
            } else {
                    universe.step();
            }
        }
    });
}
