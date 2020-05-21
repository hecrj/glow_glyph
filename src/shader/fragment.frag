#version 330

uniform sampler2D font_sampler;

in vec2 f_tex_pos;
in vec4 f_color;

out vec4 Target0;

void main() {
    float alpha = texture(font_sampler, f_tex_pos).r;

    if (alpha <= 0.0) {
        discard;
    }

    Target0 = f_color * vec4(1.0, 1.0, 1.0, alpha);
}
