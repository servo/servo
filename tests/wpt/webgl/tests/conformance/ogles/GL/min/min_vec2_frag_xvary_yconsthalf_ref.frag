
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
	const vec2 min_c = vec2(0.5, 0.5);
	vec2 c = color.rg;
	if(c[0] > min_c[0]) c[0] = min_c[0];
	if(c[1] > min_c[1]) c[1] = min_c[1];

	gl_FragColor = vec4(c, 0.0, 1.0);
}
