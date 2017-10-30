
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


/* The following files are direct copies of each other:
 *
 *   GL/acos/acos_float_frag_xvary_ref.frag
 *   GL/asin/asin_float_frag_xvary_ref.frag
 *
 * Care should be taken to apply any changes to both.  Only the last
 * line where gl_FragColor is assigned should be different.
 */

#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

float lerp(float a, float b, float s)
{
	return a + (b - a) * s;
}

void main (void)
{
	float asinValues[17];
	asinValues[0] = -1.5708;
	asinValues[1] = -1.06544;
	asinValues[2] = -0.848062;
	asinValues[3] = -0.675132;
	asinValues[4] = -0.523599;
	asinValues[5] = -0.384397;
	asinValues[6] = -0.25268;
	asinValues[7] = -0.125328;
	asinValues[8] = 0.0;
	asinValues[9] = 0.125328;
	asinValues[10] = 0.25268;
	asinValues[11] = 0.384397;
	asinValues[12] = 0.523599;
	asinValues[13] = 0.675132;
	asinValues[14] = 0.848062;
	asinValues[15] = 1.06544;
	asinValues[16] = 1.5708;
	
	const float M_PI = 3.14159265358979323846;
	float c = 2.0 * (color.r - 0.5);
	
	float arrVal = (c + 1.0) * 8.0;
	int arr0 = int(floor(arrVal));
	float weight = arrVal - floor(arrVal);
	float asin_c = 0.0;
	
	if (arr0 == 0)
		asin_c = lerp(asinValues[0], asinValues[1], weight);
	else if (arr0 == 1)
		asin_c = lerp(asinValues[1], asinValues[2], weight);
	else if (arr0 == 2)
		asin_c = lerp(asinValues[2], asinValues[3], weight);
	else if (arr0 == 3)
		asin_c = lerp(asinValues[3], asinValues[4], weight);
	else if (arr0 == 4)
		asin_c = lerp(asinValues[4], asinValues[5], weight);
	else if (arr0 == 5)
		asin_c = lerp(asinValues[5], asinValues[6], weight);
	else if (arr0 == 6)
		asin_c = lerp(asinValues[6], asinValues[7], weight);
	else if (arr0 == 7)
		asin_c = lerp(asinValues[7], asinValues[8], weight);
	else if (arr0 == 8)
		asin_c = lerp(asinValues[8], asinValues[9], weight);
	else if (arr0 == 9)
		asin_c = lerp(asinValues[9], asinValues[10], weight);
	else if (arr0 == 10)
		asin_c = lerp(asinValues[10], asinValues[11], weight);
	else if (arr0 == 11)
		asin_c = lerp(asinValues[11], asinValues[12], weight);
	else if (arr0 == 12)
		asin_c = lerp(asinValues[12], asinValues[13], weight);
	else if (arr0 == 13)
		asin_c = lerp(asinValues[13], asinValues[14], weight);
	else if (arr0 == 14)
		asin_c = lerp(asinValues[14], asinValues[15], weight);
	else if (arr0 == 15)
		asin_c = lerp(asinValues[15], asinValues[16], weight);
	else if (arr0 == 16)
		asin_c = asinValues[16];

	gl_FragColor = vec4(asin_c / M_PI + 0.5, 0.0, 0.0, 1.0);
}
