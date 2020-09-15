#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Light {
    vec3 direction;
    mat4 light_mat;
};

void main() {
    gl_Position = light_mat * vec4(position, 1.0);
}
