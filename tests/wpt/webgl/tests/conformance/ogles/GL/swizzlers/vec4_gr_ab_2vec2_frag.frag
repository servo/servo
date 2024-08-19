
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
	vec2 m = al.gr;
	vec2 n = al.ab;
	vec4 a = vec4(m.g, m.r, n.g, n.r);
	gl_FragColor = a;
}
