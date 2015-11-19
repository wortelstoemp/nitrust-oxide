#version 330 core

layout (location = 0) in vec3 position;
layout (location = 1) in vec3 color;
layout (location = 2) in vec2 uv_coordinates;

uniform mat4 transform;

out vec3 frag_color;
out vec2 frag_uv;

void main() {
   gl_Position = transform * vec4(position.x, position.y, position.z, 1.0);
   frag_color = color;
   frag_uv = uv_coordinates;
}
