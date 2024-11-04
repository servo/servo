
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
	vec3 x = 2.0 * (color.ggg - 0.5);
	vec3 y = 2.0 * (color.bbb - 0.5);
	const float epsilon = 1.0e-4;
	gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);

	// Avoid evaluating atan(0, x) for x < epsilon because it's implementation-dependent
	if(x[0] > epsilon || abs(y[0]) > epsilon)
	{
		gl_FragColor[0] = atan(y[0], x[0]) / (2.0 * M_PI) + 0.5;
	}

	if(x[1] > epsilon || abs(y[1]) > epsilon)
	{
		gl_FragColor[1] = atan(y[1], x[1]) / (2.0 * M_PI) + 0.5;
	}

	if(x[2] > epsilon || abs(y[2]) > epsilon)
	{
		gl_FragColor[2] = atan(y[2], x[2]) / (2.0 * M_PI) + 0.5;
	}
}
