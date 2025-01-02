
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
	vec4 m = al.barg;
	vec4 a = vec4(m.b, m.a, m.r, m.g);
	gl_FragColor = a;
}
