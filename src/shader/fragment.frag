#version 100
precision mediump float;

uniform sampler2D font_sampler;

varying vec2 f_uv;
varying vec4 f_color;

void main() {
    float alpha = texture2D(font_sampler, f_uv).a;
    gl_FragColor = f_color * vec4(1.0, 1.0, 1.0, alpha);
}
