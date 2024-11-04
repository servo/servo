
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct nestb
{
	mat2 b;
};

struct nesta
{
	mat2 a;
	nestb nest_b;
};

struct nest
{
	nesta nest_a;
};

void main (void)
{
	nest s = nest(nesta( mat2(11, 13, 29, 33), nestb( mat2(12, 19, 79, 81) ) ) );


	gl_FragColor = vec4( vec3(  (s.nest_a.a[0][0] + s.nest_a.a[0][1] + s.nest_a.a[1][0] + s.nest_a.a[1][1] + s.nest_a.nest_b.b[0][0] + s.nest_a.nest_b.b[0][1] + s.nest_a.nest_b.b[1][0] + s.nest_a.nest_b.b[1][1] ) / 277.0 ), 1.0);
}
