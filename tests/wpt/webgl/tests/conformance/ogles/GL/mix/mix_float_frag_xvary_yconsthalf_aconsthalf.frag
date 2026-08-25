
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
	const float y = 0.5;
	const float a = 0.5;
	float c = color.r;
	gl_FragColor = vec4(mix(c, y, a), 0.0, 0.0, 1.0);
}
