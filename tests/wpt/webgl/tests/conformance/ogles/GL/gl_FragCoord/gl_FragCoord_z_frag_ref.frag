
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 position;

void main(void)
{
	// Normalized device coordinates
	float z = position.z / position.w;
	float f = gl_DepthRange.far;
	float n = gl_DepthRange.near;

	// Window coordinates
	z = ((f - n) / 2.0) * z + (f + n) / 2.0;

	gl_FragColor = vec4(vec3(z), 1.0);
}
