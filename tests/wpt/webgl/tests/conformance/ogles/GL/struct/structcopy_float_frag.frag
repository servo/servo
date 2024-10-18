
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct sabcd
{
	float a;
	float b;
	float c;
	float d;
};



void main (void)
{
	sabcd s = sabcd(1.0, 2.0, 4.0, 8.0);
	sabcd s2 = sabcd(0.0, 0.0, 0.0, 0.0);
	s2 = s;
	gl_FragColor = vec4((s.a + s.b + s.c + s.d) / 15.0, (s2.a + s2.b + s2.c + s2.d) / 15.0, 1.0, 1.0);
}
