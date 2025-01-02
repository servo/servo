
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 gtf_TexCoord[1];
attribute vec4 gtf_MultiTexCoord0;
varying vec4 color;

void main (void)
{
	color = gtf_Color;
	gtf_TexCoord[0] = gtf_MultiTexCoord0;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
