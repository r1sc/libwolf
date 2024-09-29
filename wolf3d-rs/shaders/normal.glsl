#ifdef VS

layout(location = 0) in vec2 a_pos;
layout(location = 1) in vec2 a_uv;

out vec2 uv;

void main() {
	uv = a_uv;
	gl_Position = vec4(a_pos, 0.0, 1.0);
}
#endif

#ifdef FS

precision mediump float;
in vec2 uv;
uniform sampler2D u_framebuffer;

out vec4 color;

void main() {
	color = texture2D(u_framebuffer, uv);
}
#endif