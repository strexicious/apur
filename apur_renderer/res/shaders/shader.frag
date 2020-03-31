#version 450

layout(location = 0) in vec2 f_tex_coords;
layout(location = 1) in vec3 f_normal;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform sampler s_albedo;
layout(set = 1, binding = 0) uniform texture2D t_albedo;
layout(set = 0, binding = 2) uniform Light {
    vec3 direction;
} light;

void main() {
    // we assume a white light
    out_color = max(0.1, dot(f_normal, light.direction)) * texture(sampler2D(t_albedo, s_albedo), f_tex_coords);
}
