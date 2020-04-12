#version 450

layout(location = 0) in vec2 f_tex_coords;
layout(location = 1) in vec3 f_normal;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform sampler f_sampler;
layout(set = 0, binding = 2) uniform Light {
    vec3 direction;
} light;

layout(set = 1, binding = 0) uniform Color {
    vec3 albedo;
};
layout(set = 1, binding = 1) uniform texture2D diffuse_map;
layout(set = 1, binding = 2) uniform texture2D specular_map;

void main() {
    out_color = texture(sampler2D(specular_map, f_sampler), f_tex_coords) * max(0.0, dot(f_normal, light.direction)) * texture(sampler2D(diffuse_map, f_sampler), f_tex_coords);
}
