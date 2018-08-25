
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

/* This epsilon will work as long as the magnitude of the float is < 128.
 * This can be seen by taking the spec relative mediump precision of 2^-10:
 * 0.125 / 2^-10 = 128
 */
#define ERROR_EPSILON (0.125)

void main (void)
{
	mat4 a = mat4( 1.0, 2.0, 3.0, 4.0,
				   5.0, 6.0, 7.0, 8.0,
                   9.0, 10.0, 11.0, 12.0,
                  13.0, 14.0, 15.0, 16.0);
	float gray,sum1=0.0,sum2=0.0,sum3=0.0,sum4=0.0;
	int i;


	sum1 += a[0][0];
	sum2 += a[1][0];
	sum3 += a[2][0];
	sum4 += a[3][0];

	sum1 += a[0][1];
	sum2 += a[1][1];
	sum3 += a[2][1];
	sum4 += a[3][1];

	sum1 += a[0][2];
	sum2 += a[1][2];
	sum3 += a[2][2];
	sum4 += a[3][2];

	sum1 += a[0][3];
	sum2 += a[1][3];
	sum3 += a[2][3];
	sum4 += a[3][3];

	if( ( sum1 > 10.0-ERROR_EPSILON && sum1 < 10.0+ERROR_EPSILON  ) &&
		( sum2 > 26.0-ERROR_EPSILON && sum2 < 26.0+ERROR_EPSILON) &&
		( sum3 > 42.0-ERROR_EPSILON && sum3 < 42.0+ERROR_EPSILON) &&
		( sum4 > 58.0-ERROR_EPSILON && sum4 < 58.0+ERROR_EPSILON) )
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
