#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 interopColor;

layout(set = 0, binding = 0) uniform Props {
    vec3 offset;
};

void main() {
    interopColor = color;
    gl_Position = vec4(position + offset, 1.0);
}
