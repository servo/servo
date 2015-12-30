
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
