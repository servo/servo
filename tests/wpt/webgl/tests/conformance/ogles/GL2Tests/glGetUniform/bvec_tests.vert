
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
uniform bool vuni1;
uniform bvec2 vuni2;
uniform bvec3 vuni3;
uniform bvec4 vuni4;
varying vec4 color;

void main (void)
{
	if(vuni1 || vuni2[0] && vuni2[1] && vuni3[0] && vuni3[1] && vuni3[2] || vuni4[0] && vuni4[1] && vuni4[2] && vuni4[3])
	color = vec4(1.0, 0.0, 0.5, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
