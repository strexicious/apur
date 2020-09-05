#version 450

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Color {
    vec3 albedo;
};

void main() {
    out_color = vec4(albedo, 1.0);
}