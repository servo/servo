
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
