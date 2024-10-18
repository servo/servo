
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
	float v1 = (gtf_Color.g + 1.0) / 2.0;
	float v2 = (gtf_Color.b + 1.0) / 2.0;

	float result;
	float eta = 0.5;
	float k = 1.0 - eta * eta * (1.0 - dot(v1, v2) * dot(v1, v2));
	if(k < 0.0)
		result = 0.0;
	else
		result = eta * v1 - (eta * dot(v1, v2) + sqrt(k)) * v2;

	color = vec4((result + 1.0) / 2.0, 0.0, 0.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
