
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif
varying vec4 color;

void main (void)
{
	vec2 c = floor(10.0 * color.rg - 4.5);   // round to the nearest integer
	vec2 result = vec2(greaterThan(c, vec2(0.0)));
	gl_FragColor = vec4(result, 0.0, 1.0);
}
