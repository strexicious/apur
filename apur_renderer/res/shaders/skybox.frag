#version 450

layout(location = 0) in vec3 f_tex_coords;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform sampler s_cubemap;
layout(set = 0, binding = 1) uniform textureCube t_cubemap;

void main() {
    out_color = texture(samplerCube(t_cubemap, s_cubemap), f_tex_coords);
}
