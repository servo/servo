
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
	const float min_c = 0.25;
	const float max_c = 0.75;
	vec3 c = gtf_Color.rgb;
	if(c[0] > max_c) c[0] = max_c;
	if(c[0] < min_c) c[0] = min_c;
	if(c[1] > max_c) c[1] = max_c;
	if(c[1] < min_c) c[1] = min_c;
	if(c[2] > max_c) c[2] = max_c;
	if(c[2] < min_c) c[2] = min_c;

	color = vec4(c, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
