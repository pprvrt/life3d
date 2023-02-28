#version 150

in float v_alive;
in float v_tick;
in vec3 v_normal;
in vec3 v_position;

out vec4 color;

uniform vec3 u_light;

const vec3 ambient = vec3(0.3, 0.3, 0.3);
const vec3 diffuse = vec3(0.6, 0.6, 0.6);

vec3 ambient_color = v_alive * mix(vec3(0.0, 0.2, 0.0), ambient, v_tick) + (1.0 - v_alive) * mix(ambient, vec3(0.2, 0.0, 0.0), v_tick * 2.5);
vec3 diffuse_color = v_alive * mix(vec3(0.0, 0.6, 0.0), diffuse, v_tick) + (1.0 - v_alive) * mix(diffuse, vec3(0.6, 0.0, 0.0), v_tick * 2.5);
vec3 specular_color = vec3(1.0, 1.0, 1.0);

void main() {
    float diffuse = max(dot(normalize(v_normal), normalize(u_light)), 0.0);
    vec3 camera_dir = normalize(-v_position);
    vec3 half_direction = normalize(normalize(u_light) + camera_dir);
    float specular = pow(max(dot(half_direction, normalize(v_normal)), 0.0), 80.0);

    color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
}