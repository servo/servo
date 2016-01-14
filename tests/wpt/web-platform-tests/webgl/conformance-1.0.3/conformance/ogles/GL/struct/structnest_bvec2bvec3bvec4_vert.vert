
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

struct nestb
{
	bvec2 a2;
	bvec3 b2;
	bvec4 c2;
};

struct nesta
{
	bvec2 a1;
	bvec3 b1;
	bvec4 c1;
	nestb nest_b;
};

struct nest
{
	nesta nest_a;
};

void main (void)
{

	nest s = nest( nesta( bvec2(12, 13), bvec3(14.0, 0.0, 139.0), bvec4(25.5, 17.0, 145, 163 ), 
                       nestb( bvec2(28, 0), bvec3(0.0, 0.0, 1.0), bvec4(0.0, 17.0, 145, 0 ) 
                            ) 
                            ) 
                      );

	float gray = 0.0;

	if( ( s.nest_a.a1[0] ) && ( s.nest_a.a1[1] ) &&
            ( s.nest_a.b1[0] ) && (! (s.nest_a.b1[1]) ) && ( s.nest_a.b1[2] ) && 
            ( s.nest_a.c1[0] ) && ( s.nest_a.c1[1] ) && ( s.nest_a.c1[2] ) && ( s.nest_a.c1[3] ) && 
            ( s.nest_a.nest_b.a2[0] ) && ( !( s.nest_a.nest_b.a2[1] ) ) && 
            (! ( s.nest_a.nest_b.b2[0] ) ) && (! ( s.nest_a.nest_b.b2[1] ) ) && (s.nest_a.nest_b.b2[2]) && 
            (! ( s.nest_a.nest_b.c2[0] ) ) && (s.nest_a.nest_b.c2[1]) && (s.nest_a.nest_b.c2[2]) && (! ( s.nest_a.nest_b.c2[3] ) ) 
          )
	  gray=1.0;
	else 
          gray =0.0;
	color = vec4(gray, gray, gray, 1.0);
	
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
