#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 f_normal;
layout(location = 1) out vec3 f_view_dir;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view;
    vec3 cam_orig;
    mat4 proj;
};

void main() {
    f_normal = normal;
    f_view_dir = -(cam_orig + position);
    
    gl_Position = proj * view * vec4(position, 1.0);
}
