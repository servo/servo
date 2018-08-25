
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


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

#define ERROR_EPSILON 0.1

void main (void)
{
	float x;
	// Declare a 2 by 2 matrix with unique elements.
	mat2 a = mat2(1.0, 2.0,   // 1.0 4.0
	              4.0, 8.0);  // 2.0 8.0

	// Check each element.
	bool elms = true;
	if(a[0][0] != 1.0) elms = false;
	if(a[0][1] != 2.0) elms = false;
	if(a[1][0] != 4.0) elms = false;
	if(a[1][1] != 8.0) elms = false;

	// Add up each row.
	bool rows = true;
	x = a[0][0] + a[1][0];
	if(x < 5.0-ERROR_EPSILON || x > 5.0+ERROR_EPSILON) rows = false;
	x = a[0][1] + a[1][1];
	if(x < 10.0-ERROR_EPSILON || x > 10.0+ERROR_EPSILON) rows = false;

	// Add up each column.
	bool cols = true;
	x = a[0][0] + a[0][1];
	if(x < 3.0-ERROR_EPSILON || x > 3.0+ERROR_EPSILON) cols = false;
	x = a[1][0] + a[1][1];
	if(x < 12.0-ERROR_EPSILON || x > 12.0+ERROR_EPSILON) cols = false;

	// Check if all of the operations were successful.
	float gray = elms && rows && cols ? 1.0 : 0.0;

	// Assign the varying variable color.
	color = vec4(gray, gray, gray, 1.0);

	// Transform the vertex position.
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

