
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
