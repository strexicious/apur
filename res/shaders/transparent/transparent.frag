#version 450

layout(location = 0) out vec4 out_color;

layout(set = 1, binding = 1) uniform Color {
    vec4 albedo;
};

void main() {
    out_color = albedo;
}
