
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


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	mat2 m1 = mat2(gtf_Color.r, gtf_Color.g, gtf_Color.b, gtf_Color.a);
	mat2 m2 = mat2(1.0, 0.5, 0.5, 1.0);
	mat2 m3 = mat2(0.0);

	m3[0][0] = m1[0][0] * m2[0][0];
	m3[0][1] = m1[0][1] * m2[0][1];
	m3[1][0] = m1[1][0] * m2[1][0];
	m3[1][1] = m1[1][1] * m2[1][1];

	color = vec4(m3[0][0], m3[1][0], m3[0][1], m3[1][1]);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
