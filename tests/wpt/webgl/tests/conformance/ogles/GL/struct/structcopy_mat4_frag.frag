
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
 mat4 a;
};

void main (void)
{
	sabcd s = sabcd(mat4(12.0, 29.0, 13.0, 26.0,
			     71.0, 63.0, 90.0, 118.0,
			     128.0, 44.0, 57.0, 143.0,
			     151.0, 14.0, 15.0, 21.0 ) );
	sabcd s2 = sabcd(mat4(0.0, 0.0, 0.0, 0.0,
			     0.0, 0.0, 0.0, 0.0,
			     0.0, 0.0, 0.0, 0.0,
			     0.0, 0.0, 0.0, 0.0 ) );
	s2 = s;
	float sum=0.0;
	int i,j;

	sum = sum + s2.a[0][0];
	sum = sum + s2.a[0][1];
	sum = sum + s2.a[0][2];
	sum = sum + s2.a[0][3];
	sum = sum + s2.a[1][0];
	sum = sum + s2.a[1][1];
	sum = sum + s2.a[1][2];
	sum = sum + s2.a[1][3];
	sum = sum + s2.a[2][0];
	sum = sum + s2.a[2][1];
	sum = sum + s2.a[2][2];
	sum = sum + s2.a[2][3];
	sum = sum + s2.a[3][0];
	sum = sum + s2.a[3][1];
	sum = sum + s2.a[3][2];
	sum = sum + s2.a[3][3];

	gl_FragColor =  vec4( vec3(  sum / 995.0  ), 1.0);
}
