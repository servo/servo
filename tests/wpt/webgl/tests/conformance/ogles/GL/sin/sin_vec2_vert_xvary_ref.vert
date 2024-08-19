
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

float lerp(float a, float b, float s)
{
	return a + (b - a) * s;
}

void main (void)
{
	float sinValues[17];
	sinValues[0] = 0.0;
	sinValues[1] = 0.382683;
	sinValues[2] = 0.707107;
	sinValues[3] = 0.92388;
	sinValues[4] = 1.0;
	sinValues[5] = 0.92388;
	sinValues[6] = 0.707107;
	sinValues[7] = 0.382683;
	sinValues[8] = 0.0;
	sinValues[9] = -0.382683;
	sinValues[10] = -0.707107;
	sinValues[11] = -0.92388;
	sinValues[12] = -1.0;
	sinValues[13] = -0.923879;
	sinValues[14] = -0.707107;
	sinValues[15] = -0.382683;
	sinValues[16] = 0.0;

	const float M_PI = 3.14159265358979323846;
	vec2 c = 2.0 * M_PI * gtf_Color.rg;
	float sign = 1.0;

	vec2 sin_c = vec2(0.0);
	float fact;
	float fact_of;

	// Taylors series expansion for sin
	for(int i = 0; i < 12; i++)
	{
		fact = 1.0;
		for(int j = 2; j <= 23; j++)
			if (j <= 2 * i + 1)
				fact *= float(j);

		sin_c += sign * pow(c, vec2(2.0 * float(i) + 1.0)) / fact;
		sign *= -1.0;
	}

	color = vec4(0.5 * sin_c + 0.5, 0.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
