
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
	vec3 c = clamp((color.rgb - edge0) / (edge1 - edge0), 0.0, 1.0);

	gl_FragColor = vec4(c * c * (3.0 - 2.0 * c), 1.0);
}
