
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
attribute float att1;
attribute vec2 att2;
attribute vec3 att3;
attribute vec4 att4;
varying vec4 color;

void main (void)
{
	color = vec4(att1, att2.x + att2.y, att3.x + att3.y + att3.z, att4.x + att4.y + att4.z + att4.w);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
