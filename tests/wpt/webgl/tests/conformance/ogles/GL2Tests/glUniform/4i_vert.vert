
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
attribute vec4 gtf_Color;
uniform ivec4 color;
varying vec4 col;
void main (void)
{
	col = vec4(color);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
