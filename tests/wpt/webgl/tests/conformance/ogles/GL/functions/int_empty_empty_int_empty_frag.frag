
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
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

	gl_FragColor = vec4(gray, gray, gray, 1.0);
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
