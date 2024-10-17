
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
 mat2 a;
};

void main (void)
{
	sabcd s = sabcd(mat2(12.0, 29.0, 13.0, 26.0) );
	sabcd s2 = sabcd(mat2(0.0, 0.0, 0.0, 0.0) );
	s2 = s;
	gl_FragColor =  vec4( vec3(  (s2.a[0][0] + s2.a[0][1] + s2.a[1][0] + s2.a[1][1]) / 80.0  ), 1.0);
}
