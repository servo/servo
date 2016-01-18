
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
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
	
	gl_FragColor = result;
}

