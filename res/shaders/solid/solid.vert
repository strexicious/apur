#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view;
    vec3 cam_orig;
    mat4 proj;
};

void main() {
    gl_Position = proj * view * vec4(position, 1.0);
}
