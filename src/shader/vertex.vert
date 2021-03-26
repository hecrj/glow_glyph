#version 100

uniform mat4 transform;

attribute vec2 pos;
attribute vec2 uv;
attribute float extra;
attribute vec4 color;

varying vec2 f_uv;
varying vec4 f_color;

void main() {
    f_uv = uv;
    f_color = color;
    gl_Position = transform * vec4(pos, extra, 1.0);
}
