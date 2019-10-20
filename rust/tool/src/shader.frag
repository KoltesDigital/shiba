// precision mediump float;

/*
uniform float time;
uniform float resolutionWidth;
uniform float resolutionHeight;
*/

uniform float _[3];
#define time _[0]
#define resolutionWidth _[1]
#define resolutionHeight _[2]

#pragma fragment 0

void main() {
	vec2 uv = gl_FragCoord.xy / vec2(resolutionWidth, resolutionHeight);
	gl_FragColor = vec4(fract(uv*2.), 0., 1.0);
}
