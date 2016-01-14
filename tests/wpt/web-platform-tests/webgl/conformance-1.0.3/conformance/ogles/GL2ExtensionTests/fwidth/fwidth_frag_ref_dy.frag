
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
#extension GL_OES_standard_derivatives : enable
precision mediump float;
#endif

// setting a boundary for cases where screen sizes may exceed the precision
// of the arithmetic used.
#define SAFETY_BOUND 500.0

// Macro to scale/bias the range of output.  If input is [-1.0, 1.0], maps to [0.5, 1.0].  
// Accounts for precision errors magnified by derivative operation.
#define REDUCE_RANGE(A) ((A) + 3.0) / 4.0


uniform float viewportwidth;
uniform float viewportheight;

varying vec2 vertXY;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	float cosine;

	if( (gl_FragCoord.x < SAFETY_BOUND) && (gl_FragCoord.y < SAFETY_BOUND) )
	{
		// vertical abs cosine wave with a period of 128 pixels

#ifdef GL_OES_standard_derivatives
		cosine = REDUCE_RANGE(abs(cos(fract(gl_FragCoord.y / 128.0) * (2.0 * M_PI))));
#else
    cosine = 0.5;
#endif

		gl_FragColor = vec4(cosine, cosine, cosine, 1.0);
	}
	else discard;
}

