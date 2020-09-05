#version 450

layout(location = 0) in vec3 f_normal;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Light {
    vec3 direction;
};

void main() {
    out_color = vec4(max(0.1, dot(f_normal, -direction)) * vec3(1.0, 1.0, 0.0), 1.0);
}
