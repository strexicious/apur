#version 330

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

out vec3 interopColor;

uniform vec3 offset;

void main() {
    interopColor = color;
    gl_Position = vec4(position + offset, 1.0);
}
