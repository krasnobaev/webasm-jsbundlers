#ifdef GL_ES
precision mediump float;
#endif

#define PI 3.14159265359

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

float plot(vec2 st, float pct){
  return  smoothstep( pct-0.02, pct,      st.y ) -
          smoothstep( pct,      pct+0.02, st.y );
}

void main() {
  vec2 st = gl_FragCoord.xy/u_resolution;
  // Smooth interpolation between 0.1 and 0.9
  float y = sin(st.x + u_time * 4.0 + st.x / 50.0) / 4.0 + 0.5;

  vec3 color = vec3(y);

  float pct = plot(st,y);
  color = (1.0-pct)*1.0*vec3(1.0,1.0,1.0);

  gl_FragColor = vec4(color,0.3);
}
