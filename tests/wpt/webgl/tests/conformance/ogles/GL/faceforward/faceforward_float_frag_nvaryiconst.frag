
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
	const float M_PI = 3.14159265358979323846;

	// Both are unit vectors
	float v1 = (color.g * 2.0) - 1.0;
	float v2 = (color.b * 2.0) - 1.0;

	gl_FragColor = vec4((faceforward(v1, v2, v1) + 1.0) / 2.0, 0.0, 0.0, 1.0);
}
