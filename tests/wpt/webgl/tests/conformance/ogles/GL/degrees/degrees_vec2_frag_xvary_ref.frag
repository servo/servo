
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
	const float M_PI = 3.14159265358979323846;
	vec2 c = 2.0 * M_PI * 2.0 * (color.rg - 0.5);
	gl_FragColor = vec4((c * 180.0 / M_PI) / (2.0 * 360.0) + 0.5, 0.0, 1.0);
}
