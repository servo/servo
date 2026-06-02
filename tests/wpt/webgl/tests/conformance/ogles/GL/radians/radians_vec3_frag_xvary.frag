
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
	vec3 c = 360.0 * 2.0 * (color.rgb - 0.5);
	gl_FragColor = vec4(radians(c) / (4.0 * M_PI) + 0.5, 1.0);
}
