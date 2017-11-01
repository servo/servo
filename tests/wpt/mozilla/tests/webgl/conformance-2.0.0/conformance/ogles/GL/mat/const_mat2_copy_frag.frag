
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

/* This epsilon will work as long as the magnitude of the float is < 128.
 * This can be seen by taking the spec relative mediump precision of 2^-10:
 * 0.125 / 2^-10 = 128
 */
#define ERROR_EPSILON (0.125)

void main (void)
{
	float x;
	// Declare a constant 2 by 2 matrix with unique elements.
	const mat2 a = mat2(1.0, 2.0,   // 1.0 4.0
	                    4.0, 8.0);  // 2.0 8.0

	// Copy the constant matrix to another non-const matrix.
	mat2 b = a;

	// Check each element of the copy.
	bool elms = true;
	if(b[0][0] != 1.0) elms = false;
	if(b[0][1] != 2.0) elms = false;
	if(b[1][0] != 4.0) elms = false;
	if(b[1][1] != 8.0) elms = false;

	// Add up each row of the copy.
	bool rows = true;
	x = b[0][0] + b[1][0];
	if(x < 5.0-ERROR_EPSILON || x > 5.0+ERROR_EPSILON) rows = false;
	x = b[0][1] + b[1][1];
	if(x < 10.0-ERROR_EPSILON || x > 10.0+ERROR_EPSILON) rows = false;

	// Add up each column of the copy.
	bool cols = true;
	x = b[0][0] + b[0][1];
	if(x < 3.0-ERROR_EPSILON || x > 3.0+ERROR_EPSILON) cols = false;
	x = b[1][0] + b[1][1];
	if(x < 12.0-ERROR_EPSILON || x > 12.0+ERROR_EPSILON) cols = false;

	// Check if all of the operations were successful.
	float gray = elms && rows && cols ? 1.0 : 0.0;

	// Assign the fragment color.
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

