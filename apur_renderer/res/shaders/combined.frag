#version 450

layout(location = 0) in vec2 f_tex_coords;
layout(location = 1) in vec3 f_normal;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view;
    mat4 proj;
};
layout(set = 0, binding = 1) uniform sampler f_sampler;
layout(set = 0, binding = 2) uniform Light {
    vec3 direction;
} light;

layout(set = 1, binding = 0) uniform Color {
    float refl_idx;
};
layout(set = 1, binding = 1) uniform texture2D diffuse_map;
layout(set = 1, binding = 2) uniform texture2D specular_map;

vec3 schilcksReflection(vec3 f0, float cos_theta) {
    return f0 + (vec3(1.0) - f0) * pow(1 - cos_theta, 5.0);
}

void main() {

    // vec3 cam_pos = 
    
    // float NdV = abs(dot(f_normal, light.cam_dir));
    // float NdL = max(0.0, dot(f_normal, light.direction));
    // vec3 albedo = texture(sampler2D(diffuse_map, f_sampler), f_tex_coords).rgb;

    // vec3 m0col = NvL * schilcksReflection(vec3(0.04), NdL) + albedo;
    // vec3 m1col = NvL * schilcksReflection(albedo, NdL);

    // vec3 metalness = texture(sampler2D(specular_map, f_sampler), f_tex_coords).rgb;
    // out_color = mix(m0col, m1col, metalness);
}
