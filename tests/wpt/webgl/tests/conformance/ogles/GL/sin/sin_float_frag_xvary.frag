
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
	gl_FragColor = vec4(0.5 * sin(2.0 * M_PI * color.r) + 0.5, 0.0, 0.0, 1.0);
}
