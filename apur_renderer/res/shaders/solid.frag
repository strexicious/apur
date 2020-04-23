#version 450

layout(location = 0) in vec3 f_normal;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Light {
    vec3 direction;
    vec3 color;
} light;
layout(set = 1, binding = 0) uniform Color {
    vec3 albedo;
};

void main() {
    out_color = max(0.0, dot(f_normal, light.direction)) * vec4(albedo, 1.0);
}
