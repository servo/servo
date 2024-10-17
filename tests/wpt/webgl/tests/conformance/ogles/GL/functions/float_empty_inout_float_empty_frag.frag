
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
float function(inout float par);

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

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
float function(inout float par)
{
	// Return the value of the parameter.
	if(par == 1.0)
	{
		// Test parameter qualifier (default is "in").
		par = 0.0;

		return 1.0;
	}
	else
		return 0.0;
}
