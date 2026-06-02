
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
	float x = 2.0 * (gtf_Color.g - 0.5);
	float y = 2.0 * (gtf_Color.b - 0.5);
	float atan_c = 0.0;
	float scale = 1.0;
	float sign = 1.0;
	vec4 result = vec4(0.0, 0.0, 0.0, 1.0);
	const float epsilon = 1.0e-4;

	// Avoid evaluating atan(0, x) for x < epsilon because it's implementation-dependent
	if(x > epsilon || abs(y) > epsilon)
	{
		if(x < 0.0 ^^ y < 0.0)
		{
			sign = -1.0;
		}

		if(abs(y) <= abs(x))
		{
			float c = abs(y / x);

			// Taylors series expansion for atan
			for(int i = 1; i < 12; i += 2)
			{
				atan_c += scale * pow(c, float(i)) / float(i);
				scale *= -1.0;
			}

			result = vec4(sign * atan_c / (2.0 * M_PI) + 0.5, 0.0, 0.0, 1.0);
		}
		else
		{
			float c = abs(x / y);

			// Taylors series expansion for atan
			for(int i = 1; i < 12; i += 2)
			{
				atan_c += scale * pow(c, float(i)) / float(i);
				scale *= -1.0;
			}

			result = vec4(sign * (M_PI / 2.0 - atan_c) / (2.0 * M_PI) + 0.5, 0.0, 0.0, 1.0);
		}

		if(x < 0.0)
			if(y < 0.0) result.r -= 0.5;
			else if(y > 0.0) result.r += 0.5;
	}

	color = result;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

