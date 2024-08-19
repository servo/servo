
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
	int j = 30;
	int k = 37;
	int y = 10;
	int n = 12;
	bool result1 = false;
	bool result2 = false;
	(j>k)?( result1 = true ):( result1 = false );
	(y<n)?( result2 = true ):( result2 = false );
	float gray;
	if( !result1 && result2 )
	gray=1.0;
	else gray=0.0;
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
