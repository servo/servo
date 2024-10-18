
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
attribute float gtf_Color;

varying vec4 tc;

void main (void)
{
	tc = vec4(gtf_Color, 0.0, 0.0, 1.0);

	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}