#version 450

layout(location = 0) in vec2 f_uv;

layout(location = 0) out vec4 out_color;

struct Sphere {
    vec3 center;
    float radius;
    vec3 color;
};

layout(set = 0, binding = 0, std140) uniform Spheres {
    uint n_spheres;
    Sphere[] spheres;
};

layout(set = 0, binding = 1, std140) uniform Rendering {
    float aspect_ratio;
    float focal_length;
    vec3 cam_center;
};

void main() {
    // vec3 ray_dir = vec3(vec2(aspect_ratio * focal_length, focal_length) * f_uv, -focal_length);
    out_color = vec4(1.0, 0.0, 0.0, 1.0);
}
