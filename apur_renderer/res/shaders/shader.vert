#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec2 tex_coords;
layout(location = 2) in vec3 normal;

layout(location = 0) out vec2 f_tex_coords;
layout(location = 1) out vec3 f_normal;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view;
    mat4 proj;
};

void main() {
    f_normal = normal;
    
    f_tex_coords = tex_coords;
    f_tex_coords.y = 1.0 - f_tex_coords.y;
    
    // sponza was too big so we hardcode scale down by 100x
    gl_Position = proj * view * vec4(position / 100.0, 1.0);
    gl_Position.y = -gl_Position.y;
    gl_Position.z = (gl_Position.z + gl_Position.w) / 2.0;
}
