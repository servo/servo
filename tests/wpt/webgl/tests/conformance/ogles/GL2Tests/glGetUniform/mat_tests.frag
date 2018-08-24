
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
uniform mat2 funi2;
uniform mat3 funi3;
uniform mat4 funi4;
varying vec4 color;

void main (void)
{
	vec4 temp = vec4( funi2[0][0] + funi2[0][1] + funi2[1][0] + funi2[1][1], 

		      funi3[0][0] + funi3[0][1] + funi3[0][2] + funi3[1][0] + funi3[1][1] + funi3[1][2] + funi3[2][0] + funi3[2][1] + funi3[2][2],  

                     funi4[0][0] + funi4[0][1] + funi4[0][2] + funi4[0][3] + funi4[1][0] + funi4[1][1] + funi4[1][2] + funi4[1][3] + funi4[2][0] + funi4[2][1] + funi4[2][2] + funi4[2][3] + funi4[3][0] + funi4[3][1] + funi4[3][2] + funi4[3][3], 1.0 );
	gl_FragColor = temp + color;
}
