
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

void main (void)
{
	// Both are unit vectors
	float v1 = (color.g + 1.0) / 2.0;
	float v2 = (color.b + 1.0) / 2.0;

	float result;
	float eta = 0.5;
	float k = 1.0 - eta * eta * (1.0 - dot(v1, v2) * dot(v1, v2));
	if(k < 0.0)
		result = 0.0;
	else
		result = eta * v1 - (eta * dot(v1, v2) + sqrt(k)) * v2;

	gl_FragColor = vec4((result + 1.0) / 2.0, 0.0, 0.0, 1.0);
}
