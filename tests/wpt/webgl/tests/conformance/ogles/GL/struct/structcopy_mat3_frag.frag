
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

struct sabcd
{
 mat3 a;
};

void main (void)
{
	sabcd s = sabcd(mat3(12.0, 29.0, 13.0, 26.0, 71.0, 63.0, 90.0, 118.0, 128.0) );
	sabcd s2 = sabcd(mat3(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0) );
	s2 = s;
	float sum=0.0;
	int i,j;

	sum = sum + s2.a[0][0];
	sum = sum + s2.a[0][1];
	sum = sum + s2.a[0][2];
	sum = sum + s2.a[1][0];
	sum = sum + s2.a[1][1];
	sum = sum + s2.a[1][2];
	sum = sum + s2.a[2][0];
	sum = sum + s2.a[2][1];
	sum = sum + s2.a[2][2];

	gl_FragColor =  vec4( vec3(  sum / 550.0  ), 1.0);
}
