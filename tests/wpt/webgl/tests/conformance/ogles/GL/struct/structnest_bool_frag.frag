
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
	bool b;
};

struct nesta
{
	bool a;
	nestb nest_b;
};

struct nest
{
	nesta nest_a;
};

void main (void)
{
	nest s = nest(nesta(bool(1.0), nestb(bool(0.0))));
	float gray = 0.0;

	if( (s.nest_a.a == true) && (s.nest_a.nest_b.b == false))
	  gray=1.0;
	else
          gray =0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
