
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
bool function(bool par);

void main (void)
{
	bool par = true;
	bool ret = false;

	float gray = 0.0;

	ret = function(par);

	// The parameter should remain unchanged by the function and the function should return true.
	if(par && ret)
	{
		gray = 1.0;
	}

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
bool function(bool par)
{
	// Return the value of the parameter.
	if(par)
	{
		// Test parameter qualifier (default is "in").
		par = false;

		return true;
	}
	else
		return false;
}
