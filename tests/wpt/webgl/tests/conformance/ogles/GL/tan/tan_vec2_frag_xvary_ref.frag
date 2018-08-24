
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
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
