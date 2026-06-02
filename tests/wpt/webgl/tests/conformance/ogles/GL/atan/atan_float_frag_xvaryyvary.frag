
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif
varying vec4 color;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	float x = 2.0 * (color.g - 0.5);
	float y = 2.0 * (color.b - 0.5);
	const float epsilon = 1.0e-4;
	gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);

	// Avoid evaluating atan(0, x) for x < epsilon because it's implementation-dependent
	if(x > epsilon || abs(y) > epsilon)
	{
		gl_FragColor = vec4(atan(y, x) / (2.0 * M_PI) + 0.5, 0.0, 0.0, 1.0);
	}
}
