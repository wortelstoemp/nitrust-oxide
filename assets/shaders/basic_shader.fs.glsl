#version 330 core

in vec3 frag_color;
in vec2 frag_uv;

out vec4 out_color;

uniform sampler2D app_texture;

void main() {
   out_color = texture(app_texture, frag_uv) * vec4(frag_color, 1.0);
}
