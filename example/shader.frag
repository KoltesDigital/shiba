#version 450
precision mediump float;

uniform float time;
uniform float resolutionWidth;
uniform float resolutionHeight;

uniform sampler2D firstPassTexture;
const float PI = 3.14;
const float fallAt = 0.5;
const float startAt = 20.6;
const float continueAt = 41.8;
const float moreAt = 51.4;
const float textAt = 61;

vec4 _gl_Position;
#define gl_Position _gl_Position

#pragma shiba attributes

vec3 aPosition;
vec2 aUV;

#pragma shiba varyings

vec2 vUV;

#pragma shiba outputs

vec4 color;

#pragma shiba common

// TODO move
float beat = time * 0.78;// BPS

vec2 invAspectRatio = vec2(resolutionHeight / resolutionWidth, 1);

vec4 colorize() {
	return vec4(vec3(0.5) + vec3(0.5) * cos(2.0 * PI * (floor(beat) * 0.1 * vec3(1.0) + vec3(0.0, 1.0, 2.0) / 3.0)), 1.0);
}

mat2 rot(float a) {
	float c = cos(a), s = sin(a);
	return mat2(c, - s, s, c);
}

vec2 evoke(float t) {
	return vec2(-0.0911 + cos(t - 1.46) * 0.3 + cos(t * 2.0 - 1.04) * 0.13 + cos(t * 3-1.3) * 0.07 + cos(t * 4-1.53) * 0.07 + cos(t * 5-1.22) * 0.04 + cos(t * 6-1.02) * 0.03 + cos(t * 7-2.84) * 0.02 + cos(t * 8-1.34) * 0.03 + cos(t * 9-1.76) * 0.03 + cos(t * 10 - 1.53) * 0.03 + cos(t * 11 - 2.48) * 0.03, - 0.08 + cos(t + 0.12) * 0.07 + cos(t * 2+1.02) * 0.08 + cos(t * 3+0.66) * 0.06 + cos(t * 4+0.12) * 0.02 + cos(t * 5+0.8) * 0.05 + cos(t * 6-0.15) * 0.03 + cos(t * 7+1.1) * 0.02 + cos(t * 8+0.56) * 0.02 + cos(t * 9+0.17) * 0.06 + cos(t * 10 - 0.71) * 0.01 + cos(t * 11 + 0.47) * 0.01);
}

vec2 cookie(float t) {
	return vec2(0.08 + cos(t - 1.58) * 0.23 + cos(t * 2-1.24) * 0.14 + cos(t * 3-1.12) * 0.09 + cos(t * 4-0.76) * 0.06 + cos(t * 5-0.59) * 0.05 + cos(t * 6+0.56) * 0.03 + cos(t * 7-2.73) * 0.03 + cos(t * 8-1.26) * 0.02 + cos(t * 9-1.44) * 0.02 + cos(t * 10 - 2.09) * 0.03 + cos(t * 11 - 2.18) * 0.01 + cos(t * 12 - 1.91) * 0.02, cos(3.14) * 0.05 + cos(t + 0.35) * 0.06 + cos(t * 2+0.54) * 0.09 + cos(t * 3+0.44) * 0.03 + cos(t * 4+1.02) * 0.07 + cos(t * 6+0.39) * 0.03 + cos(t * 7-1.48) * 0.02 + cos(t * 8-3.06) * 0.02 + cos(t * 9-0.39) * 0.07 + cos(t * 10 - 0.39) * 0.03 + cos(t * 11 - 0.03) * 0.04 + cos(t * 12 - 2.08) * 0.02);
}

vec3 curve(float ratio) {
	float tt = beat;
	float ttt = 0.0;
	if (time < textAt) {
		if (time > startAt) {
			ttt += floor(beat) * 5.246;
			tt = smoothstep(0.0, fallAt + 0.15, fract(beat));
			ratio *= (4 + tt * 9);
			ratio += ttt;
		} else {
			ratio += tt;
		}
	}
	
	float radius = 0.5 + (time > continueAt ? 0.5 : 0.15) * sin(ratio);
	
	vec3 position = vec3(radius, 0, 0);
	position.xz *= rot(ttt + ratio * 1.96 * (time > moreAt ? 3:1));
	position.yz *= rot(ttt + ratio * 1.58 * (time > moreAt ? 4:1));
	position.yx *= rot(ttt + ratio * 1.5 * (time > moreAt ? 2:1));
	
	// if (time > continueAt) {
		// 	position.xz *= rot(tt * 1.5 + ttt);
		// 	position.yz *= rot(-tt * 2.0 + ttt);
	// }
	
	if (time > textAt) {
		float round = mod(floor(max(0.0, time - textAt) * 0.78), 2.0);
		if (round < 0.5)position = vec3((cookie((1 - ratio) * 2.0 * PI)) * 2, 0);
		else position = vec3((evoke((1 - ratio) * 2.0 * PI)) * 2, 0);
		// vec3 txt1 = vec3((evoke((1-ratio)*2.*PI)) * 2, 0);
		// txt1.xz *= rot(time);
		// txt1.yz *= rot(time);
	}
	
	return position * invAspectRatio.xyy;
}

float halftone(vec2 st, float dir) {
	vec2 fst = fract(st), ist = floor(st), wp = ist + step(0.5, fst), bp = ist + vec2(0.5);
	float wl = length(st - wp), bl = length(st - bp);
	return step(dir, bl / (bl + wl));
}

float floors(float x) {
	return floor(x) + smoothstep(0.9, 1.0, fract(x));
}

#pragma shiba vertex 0

// RIBBONS

void mainV0() {
	vec3 position = aPosition;
	float ratio = aUV.x;
	if (time > startAt)ratio *= smoothstep(0.0, fallAt, fract(beat));
	float size = 0.02;
	float fall = smoothstep(fallAt, 1.0, fract(beat));
	if (time > startAt)size *= smoothstep(0.1, 0.0, fall);
	position = curve(ratio);
	vec3 next = curve(ratio + 0.01);
	vec2 y = normalize(next.xy - position.xy);
	vec2 x = vec2(y.y, - y.x);
	position.xy += size * x * (aUV.y * 2.0 - 1.0) * invAspectRatio;
	position.xy /= 1.0 + position.z;
	gl_Position = vec4(position, 1.0);
}

#pragma shiba fragment 0

void mainF0() {
	color = colorize();
}

#pragma shiba vertex 1

// PARTICLES

void mainV1() {
	vec3 position = curve(aPosition.y);
	float fall = smoothstep(fallAt, 1.0, fract(beat));
	float size = (0.03 + 0.015 * sin(aPosition.y * 8654.567)) * (1.0 - fall) * smoothstep(0.0, 0.1, fall) * smoothstep(startAt - 0.5, startAt + 0.5, time);
	float a = sin(aPosition.y * 135734.2657) * PI;
	float r = sin(aPosition.y * 687451.5767) + 0.1;
	vec2 offset = vec2(cos(a), sin(a)) * r * invAspectRatio;
	offset.y -= sin(fall * PI) * 0.5 - fall * 0.5;
	position.xy += size * aUV * invAspectRatio - offset * fall;
	position.xy /= 1 + position.z;
	gl_Position = vec4(position, 1);
	vUV = aUV;
}

#pragma shiba fragment 1

void mainF1() {
	if (length(vUV) > 1.0) {
		discard;
	}
	
	color = colorize();
}

#pragma shiba fragment 2

// POST FX

void mainF2() {
	vec2 uv = gl_FragCoord.xy / vec2(resolutionWidth, resolutionHeight);
	
	color = (1.0 - colorize()) * (1.0 - 0.5 * halftone((uv - 0.5) / invAspectRatio * 40 * rot(beat / 20), floors(dot(uv, vec2(4))) / 8));

	vec4 image = texture(firstPassTexture, uv);
	color = mix(color, 0.5 * image, smoothstep(0.0, 0.001, image.a));
	
	image = texture(firstPassTexture, uv + vec2(0.01));
	color = mix(color, image, smoothstep(0.0, 0.001, image.a));
}
