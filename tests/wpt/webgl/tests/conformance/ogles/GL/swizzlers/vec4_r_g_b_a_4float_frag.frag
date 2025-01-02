
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
	vec4 al = color;
	float r = al.r;
	float g = al.g;
	float b = al.b;
	float a = al.a;
	vec4 m = vec4(r,g,b,a);
	gl_FragColor = m;
}
