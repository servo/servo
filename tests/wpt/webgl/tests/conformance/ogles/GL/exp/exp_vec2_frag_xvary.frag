
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
	const float exp3 = 20.0855;
	vec2 c = color.rg;
	gl_FragColor = vec4(exp(3.0 * c) / exp3, 0.0, 1.0);
}
