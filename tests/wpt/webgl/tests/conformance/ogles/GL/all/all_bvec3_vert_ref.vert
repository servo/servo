
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
attribute vec4 gtf_Color;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

bool _all(in bvec3 a)
{
	bool temp = true;

	if(!a[0]) temp = false;
	if(!a[1]) temp = false;
	if(!a[2]) temp = false;

	return temp;
}

void main (void)
{
	vec3 c = floor(4.0 * gtf_Color.rgb);   // 3/4 true, 1/4 false
	color = vec4(vec3(_all(bvec3(c))), 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
