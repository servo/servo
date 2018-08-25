
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

// Function declaration.
mat4 function(out mat4 par);
bool is_all(const in mat4 par, const in float value);
void set_all(out mat4 par, const in float value);

void main (void)
{
	mat4 par = mat4(1.0, 1.0, 1.0, 1.0,
			1.0, 1.0, 1.0, 1.0,
			1.0, 1.0, 1.0, 1.0,
			1.0, 1.0, 1.0, 1.0);
	mat4 ret = mat4(0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0);

	float gray = 0.0;

	ret = function(par);

	// The parameter should be changed by the function and the function should return 1.0.
	if(is_all(par, 0.0) && is_all(ret, 1.0))
	{
		gray = 1.0;
	}

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
mat4 function(out mat4 par)
{
	// Test parameter qualifier (default is "in").
	set_all(par, 0.0);

	return mat4(1.0, 1.0, 1.0, 1.0,
		    1.0, 1.0, 1.0, 1.0,
		    1.0, 1.0, 1.0, 1.0,
		    1.0, 1.0, 1.0, 1.0);
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

void set_all(out mat4 par, const in float value)
{
	par[0][0] = value;
	par[0][1] = value;
	par[0][2] = value;
	par[0][3] = value;

	par[1][0] = value;
	par[1][1] = value;
	par[1][2] = value;
	par[1][3] = value;

	par[2][0] = value;
	par[2][1] = value;
	par[2][2] = value;
	par[2][3] = value;

	par[3][0] = value;
	par[3][1] = value;
	par[3][2] = value;
	par[3][3] = value;
}
