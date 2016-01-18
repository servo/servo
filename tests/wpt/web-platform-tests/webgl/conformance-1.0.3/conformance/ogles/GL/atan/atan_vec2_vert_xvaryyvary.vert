
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
attribute vec4 gtf_Color;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	vec2 x = 2.0 * (gtf_Color.gg - 0.5);
	vec2 y = 2.0 * (gtf_Color.bb - 0.5);
	const float epsilon = 1.0e-4;
	color = vec4(0.0, 0.0, 0.0, 1.0);
	
	// Avoid evaluating atan(0, x) for x < epsilon because it's implementation-dependent
	if(x[0] > epsilon || abs(y[0]) > epsilon)
	{
		color[0] = atan(y[0], x[0]) / (2.0 * M_PI) + 0.5;
	}

	if(x[1] > epsilon || abs(y[1]) > epsilon)
	{
		color[1] = atan(y[1], x[1]) / (2.0 * M_PI) + 0.5;
	}
	
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
