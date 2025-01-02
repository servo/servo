
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

// Function declaration.
int function(int par);

void main (void)
{
	int par = 1;
	int ret = 0;

	float gray = 0.0;

	ret = function(par);

	// The parameter should remain unchanged by the function and the function should return 1.
	if((par == 1) && (ret == 1))
	{
		gray = 1.0;
	}

	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

// Function definition.
int function(int par)
{
	// Return the value of the parameter.
	if(par == 1)
	{
		// Test parameter qualifier (default is "in").
		par = 0;

		return 1;
	}
	else
		return 0;
}
