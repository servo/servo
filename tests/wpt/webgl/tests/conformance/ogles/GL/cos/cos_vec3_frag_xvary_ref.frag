
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
	vec3 c = 2.0 * M_PI * ( fract(abs(color.rgb)) - 0.5 );
	float sign =  1.0;
	vec3 cos_c = vec3(-1.0, -1.0, -1.0);
	float fact_even = 1.0;
	float fact_odd  = 1.0;
	vec3 sum;
	vec3 exp;

	// At this point c is in the range [-PI, PI)

	// Taylor-Maclaurin series expansion for cosine
	//
	// Apply the property that pow(a, b + c) = pow(a, b) * pow(a, c)
	// and the property that 1.0/(a*b) = 1.0/a * 1.0/b
	// to make sure no register ever overflows the range (-16384, +16384)
	// mandated for mediump variables.

	for(int i = 2; i <= 10; i += 2)
	{
		// fact_even will hold at most the value 3840.
		fact_even *= float(i);

		// fact_odd will always be smaller than fact_even
		fact_odd  *= float(i-1);

		// exp is at most (5,5,5)
		exp = vec3(float(i/2), float(i/2), float(i/2));

		// pow(c, exp) takes at most the value pow(PI, 5), which is approx. 306
		// abs(sum) is at most PI/2.0
		sum = sign * pow(abs(c), exp)/fact_even;

		// abs(sum/fact_odd) is at most PI/2.0
		// cos_c is always bound in the range [-1.0, 1.0)
		cos_c += pow(abs(c), exp)*(sum/fact_odd);

		sign = -sign;
	}

	gl_FragColor = vec4(0.5 * cos_c + 0.5, 1.0);
}
