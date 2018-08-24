
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



float qualifiers(in float a, out float b, inout float c, const in float d, float e)
{
	b = a;
	c += d;
	a += 1.0;
	return e;
}



void main (void)
{
	float a = 1.0, b = 2.0, c = 3.0, d = 4.0, e = 1.0, f = 0.0;
	float q = 0.0;
	float q2 = 0.0;

	f = qualifiers(a, b, c, d, e);

	if(a == 1.0) q += 1.0;
	if(b == 1.0) q += 2.0;
	if(c == 7.0) q += 4.0;
	if(d == 4.0) q2 += 1.0;
	if(e == 1.0) q2 += 2.0;
	if(f == 1.0) q2 += 4.0;

	color = vec4(vec2(q / 7.0, q2 / 7.0), 1.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
