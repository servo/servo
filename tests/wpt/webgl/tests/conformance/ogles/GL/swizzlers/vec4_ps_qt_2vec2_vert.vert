
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
	vec2 m = lightloc.ps;
	vec2 n = lightloc.qt;
	vec4 a = vec4(m.t, n.t, m.s, n.s);
	color = gtf_Color;
	gl_Position = gtf_ModelViewProjectionMatrix * a;
}
