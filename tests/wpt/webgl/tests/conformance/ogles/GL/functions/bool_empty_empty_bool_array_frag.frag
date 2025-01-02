
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

// Function declarations.
bool function(bool par[3]);
bool is_all(const in bool array[3], const in bool value);
void set_all(out bool array[3], const in bool value);

void main (void)
{
	bool par[3];
	bool ret = false;

	float gray = 0.0;

	// Initialize the entire array to true.
	set_all(par, true);

	ret = function(par);

	// The parameter should remain unchanged by the function and the function should return true.
	if(is_all(par, true) && ret)
	{
		gray = 1.0;
	}

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definitions.
bool function(bool par[3])
{
	// Return the value of the array.
	if(is_all(par, true))
	{
		// Test parameter qualifier (default is "in").
		set_all(par, false);

		return true;
	}
	else
		return false;
}

bool is_all(const in bool array[3], const in bool value)
{
	bool ret = true;

	if(array[0] != value)
		ret = false;
	if(array[1] != value)
		ret = false;
	if(array[2] != value)
		ret = false;

	return ret;
}

void set_all(out bool array[3], const in bool value)
{
	array[0] = value;
	array[1] = value;
	array[2] = value;
}
