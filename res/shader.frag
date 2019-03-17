#version 330

in vec3 interopColor;

out vec4 fragColor;

void main() {
    fragColor = vec4(interopColor, 1.0);
}
