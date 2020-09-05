#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform LightView {
    mat4 light_view;
};

void main() {
    gl_Position = light_view * vec4(position, 1.0);
}
