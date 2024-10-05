
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
attribute mat2 att2;
attribute mat3 att3;
varying vec4 color;

void main (void)
{
	color = vec4( att2[0][0] + att2[0][1] + att2[1][0] + att2[1][1], 

		      att3[0][0] + att3[0][1] + att3[0][2] + att3[1][0] + att3[1][1] + att3[1][2] + att3[2][0] + att3[2][1] + att3[2][2],  

                     1.0, 1.0 );

	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
