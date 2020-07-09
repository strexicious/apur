#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec2 f_tex_coords;
layout(location = 1) out vec3 f_normal;
layout(location = 2) out vec2 f_position;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view;
    mat4 proj;
};

void main() {
    // f_tex_coords = tex_coords;
    // f_tex_coords.y = 1.0 - f_tex_coords.y;
    // f_normal = normal;
    // f_normal.y *= -1.0;
    // f_position = position;
    
    gl_Position = proj * view * vec4(position, 1.0);
}
