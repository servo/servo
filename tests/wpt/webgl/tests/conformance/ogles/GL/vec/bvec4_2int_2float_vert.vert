
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	bvec4 a = bvec4(0, 23, 0.0, 23.0);
	float gray;
	if( (a[0] == false) && (a[1] == true) && (a[2] == false) && (a[3] == true) )
	gray=1.0;
	else gray=0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

