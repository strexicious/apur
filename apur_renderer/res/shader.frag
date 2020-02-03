#version 450

layout(location = 0) in vec3 interopColor;

layout(location = 0) out vec4 fragColor;

void main() {
    fragColor = vec4(interopColor, 1.0);
}
