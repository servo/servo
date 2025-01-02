
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
	const float edge0 = 0.25;
	const float edge1 = 0.75;
	float c = clamp((color.r - edge0) / (edge1 - edge0), 0.0, 1.0);

	gl_FragColor = vec4(c * c * (3.0 - 2.0 * c), 0.0, 0.0, 1.0);
}
