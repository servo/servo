
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
	vec4 a;
	vec4 b;
};

void main (void)
{
	sabcd s = sabcd(vec4(12.0, 29.0, 32.0, 47.0), vec4(13.0, 26.0, 38.0, 53.0 ) );
	sabcd s2 = sabcd(vec4(0.0, 0.0, 0.0, 0.0), vec4(0.0, 0.0, 0.0, 0.0 ) );
	s2 = s;
	gl_FragColor = vec4( vec3(  (s2.a[0] + s2.a[1] + s2.a[2] + s2.a[3] + s2.b[0] + s2.b[1] + s2.b[2] + s2.b[3]) / 250.0  ), 1.0);
}
