#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;

layout(location = 0) out vec3 f_normal;

layout(set = 0, binding = 0) uniform Transforms {
    mat4 view;
    mat4 proj;
};

void main() {
    f_normal = normal;
    f_normal.y *= -1.0;
    
    gl_Position = proj * view * vec4(position, 1.0);
}
