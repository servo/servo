
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
	float c = 10.0 * 2.0 * (color.r - 0.5);
	c = abs(fract(c) - 0.5) * 2.0;
	gl_FragColor = vec4(c, 0.0, 0.0, 1.0);
}
