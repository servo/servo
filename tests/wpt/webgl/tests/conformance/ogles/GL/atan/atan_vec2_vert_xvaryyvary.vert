
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
attribute vec4 gtf_Color;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	vec2 x = 2.0 * (gtf_Color.gg - 0.5);
	vec2 y = 2.0 * (gtf_Color.bb - 0.5);
	const float epsilon = 1.0e-4;
	color = vec4(0.0, 0.0, 0.0, 1.0);

	// Avoid evaluating atan(0, x) for x < epsilon because it's implementation-dependent
	if(x[0] > epsilon || abs(y[0]) > epsilon)
	{
		color[0] = atan(y[0], x[0]) / (2.0 * M_PI) + 0.5;
	}

	if(x[1] > epsilon || abs(y[1]) > epsilon)
	{
		color[1] = atan(y[1], x[1]) / (2.0 * M_PI) + 0.5;
	}

	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
