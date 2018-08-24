
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

const mat4 mat_ones = mat4(1.0, 1.0, 1.0, 1.0,
			   1.0, 1.0, 1.0, 1.0,
			   1.0, 1.0, 1.0, 1.0,
			   1.0, 1.0, 1.0, 1.0);
const mat4 mat_zeros = mat4(0.0, 0.0, 0.0, 0.0,
			    0.0, 0.0, 0.0, 0.0,
			    0.0, 0.0, 0.0, 0.0,
			    0.0, 0.0, 0.0, 0.0);

// Function declarations.
mat4 function(inout mat4 par[2]);
bool is_all(const in mat4 par, const in float value);
bool is_all(const in mat4 array[2], const in mat4 value);
void set_all(out mat4 array[2], const in mat4 value);

void main (void)
{
	mat4 par[2];
	mat4 ret = mat_zeros;

	float gray = 0.0;

	// Initialize the entire array to 1.0.
	set_all(par, mat_ones);

	ret = function(par);

	// The parameter should be changed by the function and the function should return 1.0.
	if(is_all(par, mat_zeros) && is_all(ret, 1.0))
	{
		gray = 1.0;
	}

	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

// Function definitions.
mat4 function(inout mat4 par[2])
{
	// Return the value of the array.
	if(is_all(par, mat_ones))
	{
		// Test parameter qualifier (default is "in").
		set_all(par, mat_zeros);

		return mat_ones;
	}
	else
		return mat_zeros;
}

bool is_all(const in mat4 par, const in float value)
{
	bool ret = true;

	if(par[0][0] != value)
		ret = false;
	if(par[0][1] != value)
		ret = false;
	if(par[0][2] != value)
		ret = false;
	if(par[0][3] != value)
		ret = false;

	if(par[1][0] != value)
		ret = false;
	if(par[1][1] != value)
		ret = false;
	if(par[1][2] != value)
		ret = false;
	if(par[1][3] != value)
		ret = false;

	if(par[2][0] != value)
		ret = false;
	if(par[2][1] != value)
		ret = false;
	if(par[2][2] != value)
		ret = false;
	if(par[2][3] != value)
		ret = false;

	if(par[3][0] != value)
		ret = false;
	if(par[3][1] != value)
		ret = false;
	if(par[3][2] != value)
		ret = false;
	if(par[3][3] != value)
		ret = false;

	return ret;
}

bool is_all(const in mat4 array[2], const in mat4 value)
{
	bool ret = true;

	if(array[0] != value)
		ret = false;
	if(array[1] != value)
		ret = false;

	return ret;
}

void set_all(out mat4 array[2], const in mat4 value)
{
	array[0] = value;
	array[1] = value;
}
