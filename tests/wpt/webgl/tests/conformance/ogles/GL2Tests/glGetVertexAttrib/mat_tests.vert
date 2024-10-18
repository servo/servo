
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
attribute mat3 att3;
attribute mat4 att4;
varying vec4 color;

void main (void)
{
	color = vec4( 1.0, 

		      att3[0][0] + att3[0][1] + att3[0][2] + att3[1][0] + att3[1][1] + att3[1][2] + att3[2][0] + att3[2][1] + att3[2][2],  

                     att4[0][0] + att4[0][1] + att4[0][2] + att4[0][3] + att4[1][0] + att4[1][1] + att4[1][2] + att4[1][3] + att4[2][0] + att4[2][1] + att4[2][2] + att4[2][3] + att4[3][0] + att4[3][1] + att4[3][2] + att4[3][3], 1.0 );

	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
