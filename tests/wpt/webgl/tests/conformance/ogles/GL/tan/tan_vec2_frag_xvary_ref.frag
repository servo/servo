
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
	vec2 c = 0.5 * M_PI * 2.0 * (color.rg - 0.5);
	vec2 o;
	if(abs(c.r) < 0.5)   // -45..45
		o.r = 0.5 * (sin(c.r) / cos(c.r)) + 0.5;
	else   // 45..90, -45..-90
		o.r = 0.5 * (cos(c.r) / sin(c.r)) + 0.5;

	if(abs(c.g) < 0.5)   // -45..45
		o.g = 0.5 * (sin(c.g) / cos(c.g)) + 0.5;
	else   // 45..90, -45..-90
		o.g = 0.5 * (cos(c.g) / sin(c.g)) + 0.5;

	gl_FragColor = vec4(o, 0.0, 1.0);
}
