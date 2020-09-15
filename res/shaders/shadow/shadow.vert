#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 f_normal;
layout(location = 1) out vec4 f_wpos;

layout(set = 0, binding = 0) uniform CamTransforms {
    mat4 view;
    vec3 cam_orig;
    mat4 proj;
};

void main() {
    f_normal = normal;
    f_wpos = vec4(position, 1.0);
    gl_Position = proj * view * f_wpos;
}
