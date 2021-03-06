#version 450
layout(location = 0) in vec3 a_Pos;
layout(location = 1) in vec2 a_TexCoord;
layout(location = 0) out vec2 v_TexCoord;

layout(set = 0, binding = 0) uniform Locals {
    mat4 u_Transform;
};

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_Transform * vec4(a_Pos, 1.0);
    // convert from -1,1 Z to 0,1
    gl_Position.z = 0.5 * (gl_Position.z + gl_Position.w);
}
