
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
	bvec2 a;
	bvec3 b;
	bvec4 c;
};

void main (void)
{
	sabcd s = sabcd( bvec2(12, 13), bvec3(14.0, 0.0, 139.0), bvec4(25.5, 17.0, 145, 163 ) );
	sabcd s2 = sabcd( bvec2(0, 0), bvec3(0.0, 0.0, 0.0), bvec4(0.0, 0.0, 0.0, 0.0 ) );
	s2 = s;
	float gray = 0.0;
	if( (s2.a[0]) && (s2.a[1]) && (s2.b[0]) && (!s2.b[1]) && (s2.b[2]) && (s2.c[0]) && (s2.c[1]) && (s2.c[2]) )
	  gray=1.0;
	else
          gray =0.0;

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
