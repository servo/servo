
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

// Macro to scale/bias the range of output.  If input is [-1.0, 1.0], maps to [0.5, 1.0]..  
// Accounts for precision errors magnified by derivative operation.
#define REDUCE_RANGE(A) ((A) + 3.0) / 4.0


varying vec2 vertXY;

uniform float viewportwidth;
uniform float viewportheight;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	float func;
	float funcfwidth;

#ifdef GL_OES_standard_derivatives
	// fwidth of average of horizontal and vertical sine waves with periods of 128 pixels, scaled to go from -1 to +1
	func = 0.5 * (sin(fract(gl_FragCoord.x / 128.0) * (2.0 * M_PI)) + sin(fract(gl_FragCoord.y / 128.0) * (2.0 * M_PI)));
	funcfwidth = REDUCE_RANGE((128.0 / (2.0 * M_PI)) * fwidth(func));
#else
        funcfwidth = 0.5;
#endif

	if( (gl_FragCoord.x < SAFETY_BOUND) && (gl_FragCoord.y < SAFETY_BOUND) )
	{
		gl_FragColor = vec4(funcfwidth, funcfwidth, funcfwidth, 1.0);
	}
	else discard;
}

