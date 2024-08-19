
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
	float s = al.s;
	float t = al.t;
	float p = al.p;
	float q = al.q;
	vec4 m = vec4(s,t,p,q);
	gl_FragColor = m;
}
