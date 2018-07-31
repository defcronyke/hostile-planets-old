// #version 450
// #extension GL_ARB_separate_shader_objects : enable

// layout(location = 0) in vec2 v_uv;
// layout(location = 0) out vec4 target0;

// layout(set = 0, binding = 0) uniform texture2D u_texture;
// layout(set = 0, binding = 1) uniform sampler u_sampler;

// void main() {
//   target0 = texture(sampler2D(u_texture, u_sampler), v_uv);
// }


#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;

void main() {
  outColor = vec4(v_uv.x, v_uv.y, max(v_uv.y - v_uv.x, 0.0), 1.0);
  // outColor = vec4(1.0, 0.0, 0.0, 1.0);
}
