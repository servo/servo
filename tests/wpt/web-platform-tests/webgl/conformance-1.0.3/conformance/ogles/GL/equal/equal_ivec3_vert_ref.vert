
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

bvec3 eq(in ivec3 a, in ivec3 b)
{
	bvec3 result;
	if(a[0] == b[0]) result[0] = true;
	else result[0] = false;
	if(a[1] == b[1]) result[1] = true;
	else result[1] = false;
	if(a[2] == b[2]) result[2] = true;
	else result[2] = false;
	return result;
}

void main (void)
{
	vec3 c = floor(10.0 * gtf_Color.rgb - 4.5);   // round to the nearest integer
	vec3 result = vec3(eq(ivec3(c), ivec3(0))); 
	color = vec4(result, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
