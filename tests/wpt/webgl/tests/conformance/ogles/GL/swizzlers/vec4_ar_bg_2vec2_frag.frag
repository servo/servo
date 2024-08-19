
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
	vec2 m = al.ar;
	vec2 n = al.bg;
	vec4 a = vec4(m.g, n.g, n.r, m.r);
	gl_FragColor = a;
}
