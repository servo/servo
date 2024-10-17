
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
	mat3 m = mat3(color.r, color.g, color.b, color.r, color.g, color.b, color.r, color.g, color.b);
	vec4 black = vec4(0.0, 0.0, 0.0, 1.0);
	vec4 result = vec4(1.0, 1.0, 1.0, 1.0);

	if(m[0][0] != color.r) result = black;
	if(m[0][1] != color.g) result = black;
	if(m[0][2] != color.b) result = black;
	if(m[1][0] != color.r) result = black;
	if(m[1][1] != color.g) result = black;
	if(m[1][2] != color.b) result = black;
	if(m[2][0] != color.r) result = black;
	if(m[2][1] != color.g) result = black;
	if(m[2][2] != color.b) result = black;

	gl_FragColor = result;
}
