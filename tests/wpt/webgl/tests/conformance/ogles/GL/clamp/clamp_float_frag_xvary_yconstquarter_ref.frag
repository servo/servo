
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
	const float min_c = 0.25;
	const float max_c = 0.75;
	float c = color.r;
	if(c > max_c) c = max_c;
	if(c < min_c) c = min_c;

	gl_FragColor = vec4(c, 0.0, 0.0, 1.0);
}
