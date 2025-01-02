
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
	float r = lightloc.r;
	float g = lightloc.g;
	float b = lightloc.b;
	float a = lightloc.a;
	vec4 m = vec4(r, g, b, a);
	color = gtf_Color;
	gl_Position = gtf_ModelViewProjectionMatrix * m;
}
