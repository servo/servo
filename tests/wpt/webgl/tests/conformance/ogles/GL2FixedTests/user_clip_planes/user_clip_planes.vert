
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

varying float dotClip[2];

void main (void)
{
	vec4 userClipPlanes[2];
 	userClipPlanes[0] = vec4(0.0, 1.0, 0.0, 0.0);
 	userClipPlanes[1] = vec4(-1.0, 0.0, 0.0, 0.0);

	color = gtf_Color;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
	
	dotClip[0] = dot(userClipPlanes[0], gl_Position);
	dotClip[1] = dot(userClipPlanes[1], gl_Position);
}
