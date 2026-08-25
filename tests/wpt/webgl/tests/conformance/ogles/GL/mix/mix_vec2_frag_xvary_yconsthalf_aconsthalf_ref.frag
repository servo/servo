
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

void main (void)
{
	const vec2 y = vec2(0.5, 0.5);
	const vec2 a = vec2(0.5, 0.5);
	vec2 c = color.rg;

	gl_FragColor = vec4(c * (1.0 - a) + y * a, 0.0, 1.0);
}
