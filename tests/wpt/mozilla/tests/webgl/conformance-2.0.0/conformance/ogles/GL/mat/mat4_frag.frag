
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
	// Declare a 4 by 4 matrix with unique elements.
	mat4 a = mat4( 1.0,  2.0,  3.0,  4.0,
	               5.0,  6.0,  7.0,  8.0,
	               9.0, 10.0, 11.0, 12.0,
	              13.0, 14.0, 15.0, 16.0);

	// Check each element.
	bool elms = true;
	if(a[0][0] !=  1.0) elms = false;
	if(a[0][1] !=  2.0) elms = false;
	if(a[0][2] !=  3.0) elms = false;
	if(a[0][3] !=  4.0) elms = false;
	if(a[1][0] !=  5.0) elms = false;
	if(a[1][1] !=  6.0) elms = false;
	if(a[1][2] !=  7.0) elms = false;
	if(a[1][3] !=  8.0) elms = false;
	if(a[2][0] !=  9.0) elms = false;
	if(a[2][1] != 10.0) elms = false;
	if(a[2][2] != 11.0) elms = false;
	if(a[2][3] != 12.0) elms = false;
	if(a[3][0] != 13.0) elms = false;
	if(a[3][1] != 14.0) elms = false;
	if(a[3][2] != 15.0) elms = false;
	if(a[3][3] != 16.0) elms = false;

	// Add up each row.
	bool rows = true;
	x = a[0][0] + a[1][0] + a[2][0] + a[3][0];
	if(x < 28.0-ERROR_EPSILON || x > 28.0+ERROR_EPSILON) rows = false;
	x = a[0][1] + a[1][1] + a[2][1] + a[3][1];
	if(x < 32.0-ERROR_EPSILON || x > 32.0+ERROR_EPSILON) rows = false;
	x = a[0][2] + a[1][2] + a[2][2] + a[3][2];
	if(x < 36.0-ERROR_EPSILON || x > 36.0+ERROR_EPSILON) rows = false;
	x = a[0][3] + a[1][3] + a[2][3] + a[3][3];
	if(x < 40.0-ERROR_EPSILON || x > 40.0+ERROR_EPSILON) rows = false;

	// Add up each column of the copy.
	bool cols = true;
	x = a[0][0] + a[0][1] + a[0][2] + a[0][3];
	if(x < 10.0-ERROR_EPSILON || x > 10.0+ERROR_EPSILON) cols = false;
	x = a[1][0] + a[1][1] + a[1][2] + a[1][3];
	if(x < 26.0-ERROR_EPSILON || x > 26.0+ERROR_EPSILON) cols = false;
	x = a[2][0] + a[2][1] + a[2][2] + a[2][3];
	if(x < 42.0-ERROR_EPSILON || x > 42.0+ERROR_EPSILON) cols = false;
	x = a[3][0] + a[3][1] + a[3][2] + a[3][3];
	if(x < 58.0-ERROR_EPSILON || x > 58.0+ERROR_EPSILON) cols = false;

	// Check if all of the operations were successful.
	float gray = elms && rows && cols ? 1.0 : 0.0;

	// Assign the fragment color.
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

