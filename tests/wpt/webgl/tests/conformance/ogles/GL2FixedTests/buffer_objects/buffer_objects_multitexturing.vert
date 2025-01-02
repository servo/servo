
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


uniform mat4 gtf_ModelViewProjectionMatrix;

attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
attribute vec4 gtf_MultiTexCoord0;
attribute vec4 gtf_MultiTexCoord1;

varying vec4 color;
varying vec4 gtf_TexCoord[2];

void main (void)
{
	color = gtf_Color;
	gtf_TexCoord[0] = gtf_MultiTexCoord0;
	gtf_TexCoord[1] = gtf_MultiTexCoord1;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
