
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
	const vec3 min_c = vec3(0.5, 0.5, 0.5);
	vec3 c = gtf_Color.rgb;
	if(c[0] > min_c[0]) c[0] = min_c[0];
	if(c[1] > min_c[1]) c[1] = min_c[1];
	if(c[2] > min_c[2]) c[2] = min_c[2];

	color = vec4(c, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
