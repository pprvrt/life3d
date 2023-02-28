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

/* https://github.com/glslify/glsl-easings/blob/master/bounce-out.glsl */
float bounceOut(float t) {
    const float a = 4.0 / 11.0;
    const float b = 8.0 / 11.0;
    const float c = 9.0 / 10.0;

    const float ca = 4356.0 / 361.0;
    const float cb = 35442.0 / 1805.0;
    const float cc = 16061.0 / 1805.0;

    float t2 = t * t;

    return t < a ? 7.5625 * t2 : t < b ? 9.075 * t2 - 9.9 * t + 3.4 : t < c ? ca * t2 - cb * t + cc : t > 1.0 ? 1.0 : 10.8 * t * t - 20.52 * t + 10.72;
}

void main() {
    v_alive = alive;
    v_tick = tick;

        /* Transform normal vector with model transformation matrix */
    v_normal = transpose(inverse(mat3(u_model))) * normal;

    vec4 instance = vec4(gl_InstanceID - u_width * floor(gl_InstanceID / u_width) - float(u_width) / 2.0, float(gl_InstanceID / u_width) - float(u_height) / 2.0, 0, 0);
    float wobble = alive * bounceOut(tick * 1.2) + (1.0 - alive) * (1 - smoothstep(0.0, 0.5, tick));

        /* Transform the instance according to the wobble birth&death effect */
    vec4 origin = u_model * vec4(position * wobble, 1);
        /* Move the instance on the grid, apply camera transformation and perspective transformation */
    gl_Position = u_perspective * u_view * (instance + origin);
    v_position = gl_Position.xyz / gl_Position.w;
}