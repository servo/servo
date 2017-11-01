
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

struct sabcd
{
 mat3 a;
};

void main (void)
{
	sabcd s = sabcd(mat3(12.0, 29.0, 13.0, 26.0, 71.0, 63.0, 90.0, 118.0, 128.0) );
	float sum=0.0;
	int i,j;

	sum = sum + s.a[0][0];
	sum = sum + s.a[0][1];
	sum = sum + s.a[0][2];
	sum = sum + s.a[1][0];
	sum = sum + s.a[1][1];
	sum = sum + s.a[1][2];
	sum = sum + s.a[2][0];
	sum = sum + s.a[2][1];
	sum = sum + s.a[2][2];

	gl_FragColor =  vec4( vec3(  sum / 550.0  ), 1.0);
}
