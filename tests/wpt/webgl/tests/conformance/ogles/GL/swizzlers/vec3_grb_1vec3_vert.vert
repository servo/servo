
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
	vec3 m = lightloc.rgb;
	vec3 t = m.grb;
	vec4 a = vec4(t.g, t.r, t.b, lightloc.a);
	color = gtf_Color;
	gl_Position = gtf_ModelViewProjectionMatrix * a;
}
