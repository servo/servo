
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
	vec3 c = 31.0 * gtf_Color.rgb + 1.0;
	color = vec4(log(c) / 3.466, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
