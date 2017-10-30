
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
struct nestb
{
	mat4 b;
};

struct nesta
{
	mat4 a;
	nestb nest_b;
};

struct nest
{
	nesta nest_a;
};

void main (void)
{
	nest s = nest(nesta( mat4(11, 13, 29, 33, 63, 13, 49, 57, 71, 47, 91, 101, 167, 21, 39, 41), nestb( mat4(12, 19, 79, 81, 35, 51, 73, 66, 23, 134, 121, 156, 76, 23, 24, 78) ) ) );
	float sum1=0.0,sum2=0.0;
	int i,j;
	
	sum1 = sum1 + s.nest_a.a[0][0];
	sum2 = sum2 + s.nest_a.nest_b.b[0][0];
	
	sum1 = sum1 + s.nest_a.a[0][1];
	sum2 = sum2 + s.nest_a.nest_b.b[0][1];
	
	sum1 = sum1 + s.nest_a.a[0][2];
	sum2 = sum2 + s.nest_a.nest_b.b[0][2];
	
	sum1 = sum1 + s.nest_a.a[0][3];
	sum2 = sum2 + s.nest_a.nest_b.b[0][3];
	
	sum1 = sum1 + s.nest_a.a[1][0];
	sum2 = sum2 + s.nest_a.nest_b.b[1][0];
	
	sum1 = sum1 + s.nest_a.a[1][1];
	sum2 = sum2 + s.nest_a.nest_b.b[1][1];
	
	sum1 = sum1 + s.nest_a.a[1][2];
	sum2 = sum2 + s.nest_a.nest_b.b[1][2];
	
	sum1 = sum1 + s.nest_a.a[1][3];
	sum2 = sum2 + s.nest_a.nest_b.b[1][3];
	
	sum1 = sum1 + s.nest_a.a[2][0];
	sum2 = sum2 + s.nest_a.nest_b.b[2][0];
	
	sum1 = sum1 + s.nest_a.a[2][1];
	sum2 = sum2 + s.nest_a.nest_b.b[2][1];
	
	sum1 = sum1 + s.nest_a.a[2][2];
	sum2 = sum2 + s.nest_a.nest_b.b[2][2];
	
	sum1 = sum1 + s.nest_a.a[2][3];
	sum2 = sum2 + s.nest_a.nest_b.b[2][3];
	
	sum1 = sum1 + s.nest_a.a[3][0];
	sum2 = sum2 + s.nest_a.nest_b.b[3][0];
	
	sum1 = sum1 + s.nest_a.a[3][1];
	sum2 = sum2 + s.nest_a.nest_b.b[3][1];
	
	sum1 = sum1 + s.nest_a.a[3][2];
	sum2 = sum2 + s.nest_a.nest_b.b[3][2];
	
	sum1 = sum1 + s.nest_a.a[3][3];
	sum2 = sum2 + s.nest_a.nest_b.b[3][3];

	gl_FragColor = vec4( vec3( ( sum1 + sum2 )/ 1897.0 ), 1.0);
}
