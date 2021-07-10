
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
precision mediump float;
#endif
varying vec4 color;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	vec2 c = 4.0 * 2.0 * (color.rg - 0.5);
	vec2 atan_c = vec2(0.0);
	vec2 scale = vec2(1.0);
	vec2 sign = vec2(1.0);
	vec4 result = vec4(0.0, 0.0, 0.0, 1.0);

	if(c[0] < 0.0)
	{
		sign[0] = -1.0;
		c[0] *= -1.0;
	}

	if(c[0] <= 1.0)
	{
		// Taylors series expansion for atan
		atan_c[0] += scale[0] * pow(c[0], float(1)) / float(1);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(3)) / float(3);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(5)) / float(5);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(7)) / float(7);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(9)) / float(9);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(11)) / float(11);
		scale[0] *= -1.0;

		result[0] = sign[0] * atan_c[0] / M_PI + 0.5;
	}
	else
	{
		c[0] = 1.0 / c[0];

		// Taylors series expansion for atan
		atan_c[0] += scale[0] * pow(c[0], float(1)) / float(1);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(3)) / float(3);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(5)) / float(5);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(7)) / float(7);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(9)) / float(9);
		scale[0] *= -1.0;
		atan_c[0] += scale[0] * pow(c[0], float(11)) / float(11);
		scale[0] *= -1.0;

		result[0] = sign[0] * (M_PI / 2.0 - atan_c[0]) / M_PI + 0.5;
	}


	if(c[1] < 0.0)
	{
		sign[1] = -1.0;
		c[1] *= -1.0;
	}

	if(c[1] <= 1.0)
	{
		// Taylors series expansion for atan
		atan_c[1] += scale[1] * pow(c[1], float(1)) / float(1);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(3)) / float(3);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(5)) / float(5);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(7)) / float(7);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(9)) / float(9);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(11)) / float(11);
		scale[1] *= -1.0;

		result[1] = sign[1] * atan_c[1] / M_PI + 0.5;
	}
	else
	{
		c[1] = 1.0 / c[1];

		// Taylors series expansion for atan
		atan_c[1] += scale[1] * pow(c[1], float(1)) / float(1);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(3)) / float(3);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(5)) / float(5);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(7)) / float(7);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(9)) / float(9);
		scale[1] *= -1.0;
		atan_c[1] += scale[1] * pow(c[1], float(11)) / float(11);
		scale[1] *= -1.0;

		result[1] = sign[1] * (M_PI / 2.0 - atan_c[1]) / M_PI + 0.5;
	}

	gl_FragColor = result;
}
