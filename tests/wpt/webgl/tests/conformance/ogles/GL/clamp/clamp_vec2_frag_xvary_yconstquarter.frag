
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
	const vec2 min_c = vec2(0.25, 0.25);
	const vec2 max_c = vec2(0.75, 0.75);
	vec2 c = color.rg;
	gl_FragColor = vec4(clamp(c, min_c, max_c), 0.0, 1.0);
}
