uniform mat4 transform;

in vec2 pos;
in vec2 uv;
in float extra;
in vec4 color;

out vec2 f_uv;
out vec4 f_color;

void main() {
    f_uv = uv;
    f_color = color;
    gl_Position = transform * vec4(pos, extra, 1.0);
}
