precision mediump float;

uniform float time;
uniform float resolutionWidth;
uniform float resolutionHeight;

#pragma fragment 0

// This is the template at https://www.shadertoy.com/new.
void main() {
	// Normalized pixel coordinates (from 0 to 1).
	vec2 uv = gl_FragCoord.xy / vec2(resolutionWidth, resolutionHeight);
	
	// Time varying pixel color.
	vec3 col = 0.5 + 0.5 * cos(time + uv.xyx + vec3(0, 2, 4));
	
	// Output to screen.
	gl_FragColor = vec4(col, 1.0);
}
