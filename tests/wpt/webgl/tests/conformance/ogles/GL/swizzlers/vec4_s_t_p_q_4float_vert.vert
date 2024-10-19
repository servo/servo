
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	vec4 lightloc = gtf_Vertex;
	float s = lightloc.s;
	float t = lightloc.t;
	float p = lightloc.p;
	float q = lightloc.q;
	vec4 m = vec4(s, t, p, q);
	color = gtf_Color;
	gl_Position = gtf_ModelViewProjectionMatrix * m;
}
