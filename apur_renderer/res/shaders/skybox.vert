#version 450

layout(location = 0) in vec3 position;

layout(location = 0) out vec3 f_tex_coords;

layout(set = 0, binding = 2) uniform Transforms {
    mat4 view;
    mat4 proj;
};

void main() {
    f_tex_coords = position * vec3(1.0, -1.0, 1.0);
    // we ignore the camera translation because we treat the position
    // of the cube in cam position but not the rotation
    mat4 fixed_view = view;
    fixed_view[3] = vec4(0.0, 0.0, 0.0, view[3][3]);
    gl_Position = proj * fixed_view * vec4(position, 1.0);
}
