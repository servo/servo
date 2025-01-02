
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;

varying vec2 vertXY;

void main (void)
{
	vertXY = gtf_Vertex.xy;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}


