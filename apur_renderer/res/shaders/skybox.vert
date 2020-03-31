#version 450

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 f_tex_coords;

layout(set = 0, binding = 2) uniform Transforms {
    mat4 view;
    mat4 proj;
};

void main() {
    f_tex_coords = position * vec3(1.0, -1.0, 1.0);
    gl_Position = proj * view * vec4(position, 1.0);
}
