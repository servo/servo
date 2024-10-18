
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
	float b;
};

struct nesta
{
	float a;
	nestb nest_b;
};

struct nest
{
	nesta nest_a;
};

void main (void)
{
	nest s = nest(nesta(1.0, nestb(2.0)));
	color = vec4(vec3((s.nest_a.a + s.nest_a.nest_b.b) / 3.0), 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
