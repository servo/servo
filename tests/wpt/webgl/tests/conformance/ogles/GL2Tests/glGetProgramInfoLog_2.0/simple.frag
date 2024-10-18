
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec3 tc;

void main (void)
{
	vec3 foo = tc;
	gl_FragColor = vec4 (foo, 1.0);
}