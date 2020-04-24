#version 450

layout(location = 0) in vec3 f_normal;
layout(location = 1) in vec3 f_view_dir;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 1) uniform Light {
    vec3 direction;
    vec3 color;
} light;

layout(set = 1, binding = 0) uniform Color {
    vec3 albedo;
    float roughness;
};

const float PI = 3.141592;

float beckndf(vec3 m) {
    float ndm = dot(f_normal, m);
    float roughness2 = pow(roughness,2);
    if (ndm > 0) {
        float ndm2 = pow(ndm,2);
        float expterm = exp((ndm2-1)/(roughness2*ndm2));
        return expterm / (PI*roughness2*pow(ndm2,2));
    }

    return 0;
}

float bigLambda(vec3 v) {
    float ndv = dot(f_normal,v);
    float a = ndv/(roughness*sqrt(1-pow(ndv,2)));
    if (a < 1.6) {
        return (1-1.259*a+0.396*pow(a,2)) / (3.535*a+2.181*pow(a,2));
    }
    
    return 0;
}

float smithMask(vec3 m, vec3 v) {
    float mdv = dot(m, v);
    if (mdv > 0) {
        return 1 / (1 + bigLambda(v));
    }
    
    return 0;
}

float shadowMask(vec3 l, vec3 v, vec3 m) {
    return smithMask(v, m) * smithMask(l, m);
}

vec3 schilcksReflection(float cos_theta) {
    return albedo + (vec3(1.0) - albedo) * pow(1 - cos_theta, 5.0);
}

vec3 spcular_brdf(vec3 l, vec3 v) {
    vec3 h = normalize(l + v);
    //  * beckndf(h)
    return schilcksReflection(dot(h, l)) * shadowMask(l, v, h) / (4 * abs(dot(f_normal, l)) * abs(dot(f_normal, v)));
}

void main() {
    out_color = vec4(PI * spcular_brdf(-light.direction, f_view_dir) * light.color * max(0.0, dot(f_normal, -light.direction)), 1.0);
}
