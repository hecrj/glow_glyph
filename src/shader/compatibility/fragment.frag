#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif

#ifdef HIGHER_THAN_300
out vec4 fragColor;
#define gl_FragColor fragColor
#else
#define texture texture2D
#endif

uniform sampler2D font_sampler;

in vec2 f_uv;
in vec4 f_color;

void main() {
    float alpha = texture(font_sampler, f_uv).a;
    gl_FragColor = f_color * vec4(1.0, 1.0, 1.0, alpha);
}
