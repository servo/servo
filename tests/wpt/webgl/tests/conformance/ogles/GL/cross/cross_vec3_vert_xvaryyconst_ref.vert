
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
	const float M_PI = 3.14159265358979323846;

	// Both are unit vectors
	vec3 v1;
	vec3 v2 = vec3(1.0, 0.0, 0.0);
	vec3 v3;

	float theta = gtf_Color.g * 2.0 * M_PI;
	float phi = gtf_Color.b * 2.0 * M_PI;
	v1.x = cos(theta) * sin(phi);
	v1.y = sin(theta) * sin(phi);
	v1.z = cos(phi);
	v3.x = v1.y * v2.z - v2.y * v1.z;
	v3.y = v2.x * v1.z - v1.x * v2.z;
	v3.z = v1.x * v2.y - v2.x * v1.y;
	color = vec4((v3 + 1.0) / 2.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
