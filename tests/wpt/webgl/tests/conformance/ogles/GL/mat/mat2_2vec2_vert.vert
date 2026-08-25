
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
	mat2 m = mat2(gtf_Color.rg, gtf_Color.ba);
	vec4 black = vec4(0.0, 0.0, 0.0, 1.0);
	vec4 result = vec4(1.0, 1.0, 1.0, 1.0);


	if(m[0][0] != gtf_Color.r) result = black;
	if(m[0][1] != gtf_Color.g) result = black;
	if(m[1][0] != gtf_Color.b) result = black;
	if(m[1][1] != gtf_Color.a) result = black;

	color = result;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
