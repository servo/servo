
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

// Function declaration.
float function(out float par);

void main (void)
{
	float par = 1.0;
	float ret = 0.0;

	float gray = 0.0;

	ret = function(par);

	// The parameter should be changed by the function and the function should return 1.0.
	if((par == 0.0) && (ret == 1.0))
	{
		gray = 1.0;
	}

	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

// Function definition.
float function(out float par)
{
	// Test parameter qualifier (default is "in").
	par = 0.0;

	return 1.0;
}
