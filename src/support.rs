extern crate glium;
extern crate nalgebra as na;

use crate::engine::Mouse;
use crate::universe::Universe;
use std::f32::consts::PI;

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

pub fn mouse_projection(
    target: &impl glium::Surface,
    mouse: &Mouse,
    camera: &Camera,
    perspective: &na::Perspective3<f32>,
    universe: &Universe,
) -> Option<[usize; 2]> {

    let (width, height) = target.get_dimensions();
    let ray = na::Vector3::new(
        2.0 * mouse.x() as f32 / width as f32 - 1.0,
        1.0 - 2.0 * mouse.y() as f32 / height as f32,
        0.0,
    )
    .to_homogeneous();

    let (u_width, u_height) = universe.dimensions();

    let mut ray_eye = perspective.inverse() * ray;
    (ray_eye.z, ray_eye.w) = (-1.0, 0.0);

    let mut ray_world = (camera.matrix.inverse().to_homogeneous() * ray_eye).xyz();
    ray_world.normalize_mut();

    let t = -camera.position[2] / ray_world[2];
    let x = camera.position[0] + ray_world[0] * t + 0.5 + u_width as f32 / 2.0;
    let y = camera.position[1] + ray_world[1] * t + 0.5 + u_height as f32 / 2.0;

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

pub fn vertex_shader() -> &'static str {
    r#"
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
    "#
}

pub fn fragment_shader() -> &'static str {
    r#"
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
    "#
}
