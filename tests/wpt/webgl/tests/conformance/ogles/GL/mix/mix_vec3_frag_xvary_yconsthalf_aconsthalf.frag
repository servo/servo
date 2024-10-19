
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
	const vec3 y = vec3(0.5, 0.5, 0.5);
	const vec3 a = vec3(0.5, 0.5, 0.5);
	gl_FragColor = vec4(mix(color.rgb, y, a), 1.0);
}
