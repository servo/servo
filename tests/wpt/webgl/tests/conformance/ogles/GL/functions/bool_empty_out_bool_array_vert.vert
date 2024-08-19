
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

// Function declaration.
bool function(out bool par[3]);
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

	// The parameter should be changed by the function and the function should return true.
	if(is_all(par, false) && ret)
	{
		gray = 1.0;
	}

	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

// Function definition.
bool function(out bool par[3])
{
	// Test parameter qualifier (default is "in").
	set_all(par, false);

	return true;
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
