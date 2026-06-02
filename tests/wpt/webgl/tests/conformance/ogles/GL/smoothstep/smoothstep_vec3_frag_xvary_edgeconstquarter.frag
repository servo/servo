
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
	const vec3 edge0 = vec3(0.25, 0.25, 0.25);
	const vec3 edge1 = vec3(0.75, 0.75, 0.75);
	gl_FragColor = vec4(smoothstep(edge0, edge1, color.rgb), 1.0);
}
