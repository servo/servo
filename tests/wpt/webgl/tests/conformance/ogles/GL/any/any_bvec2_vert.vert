
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
attribute vec4 gtf_Color;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	vec2 c = floor(1.5 * gtf_Color.rg);   // 1/3 true, 2/3 false
	color = vec4(vec3(any(bvec2(c))), 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
