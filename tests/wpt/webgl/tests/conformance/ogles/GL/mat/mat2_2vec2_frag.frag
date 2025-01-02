
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
	mat2 m = mat2(color.rg, color.ba);
	vec4 black = vec4(0.0, 0.0, 0.0, 1.0);
	vec4 result = vec4(1.0, 1.0, 1.0, 1.0);

	if(m[0][0] != color.r) result = black;
	if(m[0][1] != color.g) result = black;
	if(m[1][0] != color.b) result = black;
	if(m[1][1] != color.a) result = black;

	gl_FragColor = result;
}
