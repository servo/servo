
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

void main (void)
{
	const float M_PI = 3.14159265358979323846;

	// Both are unit vectors
	vec3 v1;
	vec3 v2 = vec3(1.0, 0.0, 0.0);
	vec3 v3;

	float theta = color.g * 2.0 * M_PI;
	float phi = color.b * 2.0 * M_PI;
	v1.x = cos(theta) * sin(phi);
	v1.y = sin(theta) * sin(phi);
	v1.z = cos(phi);
	v3.x = v1.y * v2.z - v2.y * v1.z;
	v3.y = v2.x * v1.z - v1.x * v2.z;
	v3.z = v1.x * v2.y - v2.x * v1.y;
	gl_FragColor = vec4((v3 + 1.0) / 2.0, 1.0);
}
