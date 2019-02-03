#ifdef GL_ES
precision mediump float;
#endif

attribute vec2 a_position;
attribute vec4 a_scale;
varying vec4 vColor;

uniform vec2 u_resolution;
uniform vec2 u_translation;

void main() {
  // Add in the translation.
  vec2 position = a_position + u_translation;

  // convert the position from pixels to 0.0 to 1.0
  vec2 zeroToOne = position / u_resolution;

  // convert from 0->1 to 0->2
  vec2 zeroToTwo = zeroToOne * 2.0;

  // convert from 0->2 to -1->+1 (clipspace)
  vec2 clipSpace = zeroToTwo - 1.0;

  gl_Position = vec4(clipSpace * vec2(1, -1), 0, 1);
  vColor = a_scale;
}
