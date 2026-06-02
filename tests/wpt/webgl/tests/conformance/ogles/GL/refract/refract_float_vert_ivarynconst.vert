
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
	// Both are unit vectors
	float v1 = (gtf_Color.g + 1.0) / 2.0;
	float v2 = (gtf_Color.b + 1.0) / 2.0;

	color = vec4((refract(v1, v2, 0.5) + 1.0) / 2.0, 0.0, 0.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
