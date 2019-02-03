#ifdef GL_ES
precision mediump float;
#endif

uniform vec4 u_color;
varying vec4 vColor;

void main() {
  gl_FragColor = vec4(
    u_color.x + u_color.x - 0.5,
    u_color.y + u_color.y - 0.5,
    u_color.z + u_color.z - 0.5,
    u_color.w + u_color.w - 0.5
  );
}
