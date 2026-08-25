
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
	vec2 c = -gtf_Color.rg;
	color = vec4(exp(3.0 * c), 0.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
