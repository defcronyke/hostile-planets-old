#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(constant_id = 0) const float scale = 1.2f;

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;
layout(location = 0) out vec2 v_uv;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    v_uv = a_uv;
    gl_Position = vec4(scale * a_pos, 0.0, 1.0);
}


// #version 450
// #extension GL_ARB_separate_shader_objects : enable

// layout(constant_id = 0) const float scale = 1.2f;

// layout(location = 0) in vec3 a_pos;
// // layout(location = 0) in vec2 a_pos;
// layout(location = 1) in vec2 a_uv;

// layout(binding = 2) uniform UniformBufferObject {
//     mat4 model;
//     mat4 view;
//     mat4 proj;
//     mat4 clip;
// } ubo;

// layout(location = 0) out vec2 v_uv;


// out gl_PerVertex {
//     vec4 gl_Position;
// };

// void main() {
//     v_uv = a_uv;
//     gl_Position = ubo.clip * ubo.proj * ubo.view * ubo.model * vec4(a_pos, 1.0);
//     // gl_Position = ubo.clip * ubo.proj * ubo.view * ubo.model * vec4(scale * a_pos, 1.0);
//     // gl_Position = vec4(scale * a_pos, 1.0);
// }
