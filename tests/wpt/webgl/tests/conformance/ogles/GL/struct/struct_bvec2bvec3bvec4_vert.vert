
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

struct sabcd
{
	bvec2 a;
	bvec3 b;
	bvec4 c;
};

void main (void)
{
	sabcd s = sabcd( bvec2(12, 13), bvec3(14.0, 0.0, 139.0), bvec4(25.5, 17.0, 145, 163 ) );
	float gray = 0.0;
	if( (s.a[0]) && (s.a[1]) && (s.b[0]) && (!s.b[1]) && (s.b[2]) && (s.c[0]) && (s.c[1]) && (s.c[2]) )
	  gray=1.0;
	else
          gray =0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
