
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;



float qualifiers(in float a, out float b, inout float c, const in float d, float e)
{
	b = a;
	c += d;
	a += 1.0;
	return e;
}



void main (void)
{
	float a = 1.0, b = 2.0, c = 3.0, d = 4.0, e = 1.0, f = 0.0;
	float q = 0.0;
	float q2 = 0.0;

	f = qualifiers(a, b, c, d, e);

	if(a == 1.0) q += 1.0;
	if(b == 1.0) q += 2.0;
	if(c == 7.0) q += 4.0;
	if(d == 4.0) q2 += 1.0;
	if(e == 1.0) q2 += 2.0;
	if(f == 1.0) q2 += 4.0;

	gl_FragColor = vec4(vec2(q / 7.0, q2 / 7.0), 1.0, 1.0);
}
