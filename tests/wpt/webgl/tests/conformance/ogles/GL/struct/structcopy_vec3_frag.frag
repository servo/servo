
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
	vec3 a;
	vec3 b;
};


void main (void)
{
	sabcd s = sabcd(vec3(12.0, 29.0, 32.0), vec3(13.0, 26.0, 38.0 ) );
	sabcd s2 = sabcd(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, 0.0 ) );
	s2 = s;
	gl_FragColor = vec4( vec3(  (s2.a[0] + s2.a[1] + s2.a[2] + s2.b[0] + s2.b[1] + s2.b[2]) / 150.0  ), 1.0);
}
