#version 450

layout(location = 0) in vec2 f_tex_coords;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform texture2D t_albedo;
layout(set = 0, binding = 2) uniform sampler s_albedo;

void main() {
    out_color = texture(sampler2D(t_albedo, s_albedo), f_tex_coords);
}
