
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
varying vec4 color;

const int array_size = 2;

void main (void)
{
	const mat3 a = mat3(1.0, 2.0, 3.0,
		            4.0, 5.0, 6.0,
			    7.0, 8.0, 9.0);
	const mat3 b = mat3(10.0, 11.0, 12.0,
		            13.0, 14.0, 15.0,
			    16.0, 17.0, 18.0);
	mat3 array[array_size];
	float gray;

	array[0] = a;
	array[1] = b;

	if((array[0] == a) && (array[1] == b))
		gray = 1.0;
	else
		gray = 0.0;

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

