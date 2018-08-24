
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
int function(inout int par);

void main (void)
{
	int par = 1;
	int ret = 0;

	float gray = 0.0;

	ret = function(par);

	// The parameter should be changed by the function and the function should return 1.
	if((par == 0) && (ret == 1))
	{
		gray = 1.0;
	}

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
int function(inout int par)
{
	// Return the value of the parameter.
	if(par == 1)
	{
		// Test parameter qualifier (default is "in").
		par = 0;

		return 1;
	}
	else
		return 0;
}
