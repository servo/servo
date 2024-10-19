
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
	vec3 c = 2.0 * (gtf_Color.rgb - 0.5);
	if((c[0] < 0.0)) c[0] *= -1.0;
	if((c[1] < 0.0)) c[1] *= -1.0;
	if((c[2] < 0.0)) c[2] *= -1.0;

	color = vec4(c, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
