
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
	int i=0;
	float new_mad[2];
	float gray = 0.0;

	new_mad[0]=float(1);
	new_mad[1]=float(2);

	if( (new_mad[0] == 1.0) && (new_mad[1] == 2.0) )
	  gray=1.0;
	else gray=0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
