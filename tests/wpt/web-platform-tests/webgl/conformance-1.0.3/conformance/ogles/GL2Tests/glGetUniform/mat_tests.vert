
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
uniform mat2 vuni2;
uniform mat3 vuni3;
uniform mat4 vuni4;
varying vec4 color;

void main (void)
{
	color = vec4( vuni2[0][0] + vuni2[0][1] + vuni2[1][0] + vuni2[1][1], 

		      vuni3[0][0] + vuni3[0][1] + vuni3[0][2] + vuni3[1][0] + vuni3[1][1] + vuni3[1][2] + vuni3[2][0] + vuni3[2][1] + vuni3[2][2],  

                     vuni4[0][0] + vuni4[0][1] + vuni4[0][2] + vuni4[0][3] + vuni4[1][0] + vuni4[1][1] + vuni4[1][2] + vuni4[1][3] + vuni4[2][0] + vuni4[2][1] + vuni4[2][2] + vuni4[2][3] + vuni4[3][0] + vuni4[3][1] + vuni4[3][2] + vuni4[3][3], 1.0 );

	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
