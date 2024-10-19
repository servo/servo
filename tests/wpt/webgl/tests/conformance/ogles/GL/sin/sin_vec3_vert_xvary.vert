
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
	const float M_PI = 3.14159265358979323846;
	color = vec4(0.5 * sin(2.0 * M_PI * gtf_Color.rgb) + 0.5, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
