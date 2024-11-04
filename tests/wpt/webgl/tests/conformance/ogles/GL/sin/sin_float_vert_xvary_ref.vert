
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	float c = 2.0 * M_PI * gtf_Color.r;
	float sign = 1.0;

	float sin_c = 0.0;
	float fact;
	float fact_of;

	// Taylors series expansion for sin
	for(int i = 0; i < 12; i++)
	{
		fact = 1.0;
		for(int j = 2; j <= 23; j++)
			if (j <= 2 * i + 1)
				fact *= float(j);

		sin_c += sign * pow(c, 2.0 * float(i) + 1.0) / fact;
		sign *= -1.0;
	}

	color = vec4(0.5 * sin_c + 0.5, 0.0, 0.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
