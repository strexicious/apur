#version 450

layout(location = 0) in vec3 f_normal;
layout(location = 1) in vec4 f_wpos;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Light {
    vec3 direction;
    mat4 light_transform;
};
layout(set = 0, binding = 2) uniform sampler s_nn_r; // nearest-neighbour, repeat
layout(set = 0, binding = 3) uniform texture2D t_shadow_map;

void main() {
    vec4 lpos = light_transform * f_wpos;
    vec2 smcoords = lpos.xy * vec2(0.5, -0.5) + vec2(0.5, 0.5);
    float lit = texture(sampler2DShadow(t_shadow_map, s_nn_r), vec3(smcoords, lpos.z)) * 0.8 + 0.2;

    out_color = vec4(max(0.1, dot(f_normal, -direction)) * vec3(1.0, 1.0, 0.0) * lit, 1.0);
}
