
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

struct nestb
{
	mat3 b;
};

struct nesta
{
	mat3 a;
	nestb nest_b;
};

struct nest
{
	nesta nest_a;
};

void main (void)
{
	nest s = nest(nesta( mat3(11, 13, 29, 33, 63, 13, 49, 57, 71), nestb( mat3(12, 19, 79, 81, 35, 51, 73, 66, 23) ) ) );
	float sum1=0.0,sum2=0.0;

	sum1 = sum1 + s.nest_a.a[0][0];
	sum2 = sum2 + s.nest_a.nest_b.b[0][0];
	sum1 = sum1 + s.nest_a.a[0][1];
	sum2 = sum2 + s.nest_a.nest_b.b[0][1];
	sum1 = sum1 + s.nest_a.a[0][2];
	sum2 = sum2 + s.nest_a.nest_b.b[0][2];

	sum1 = sum1 + s.nest_a.a[1][0];
	sum2 = sum2 + s.nest_a.nest_b.b[1][0];
	sum1 = sum1 + s.nest_a.a[1][1];
	sum2 = sum2 + s.nest_a.nest_b.b[1][1];
	sum1 = sum1 + s.nest_a.a[1][2];
	sum2 = sum2 + s.nest_a.nest_b.b[1][2];

	sum1 = sum1 + s.nest_a.a[2][0];
	sum2 = sum2 + s.nest_a.nest_b.b[2][0];
	sum1 = sum1 + s.nest_a.a[2][1];
	sum2 = sum2 + s.nest_a.nest_b.b[2][1];
	sum1 = sum1 + s.nest_a.a[2][2];
	sum2 = sum2 + s.nest_a.nest_b.b[2][2];

	color = vec4( vec3( ( sum1 + sum2 )/ 778.0 ), 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
