
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

// Function declarations.
int function(inout int par[3]);
bool is_all(const in int array[3], const in int value);
void set_all(out int array[3], const in int value);

void main (void)
{
	int par[3];
	int ret = 0;

	float gray = 0.0;

	// Initialize the entire array to 1.
	set_all(par, 1);

	ret = function(par);

	// The parameter should be changed by the function and the function should return 1.
	if(is_all(par, 0) && (ret == 1))
	{
		gray = 1.0;
	}

	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

// Function definition.
int function(inout int par[3])
{
	// Return the value of the array.
	if(is_all(par, 1))
	{
		// Test parameter qualifier (default is "in").
		set_all(par, 0);

		return 1;
	}
	else
		return 0;
}

bool is_all(const in int array[3], const in int value)
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

void set_all(out int array[3], const in int value)
{
	array[0] = value;
	array[1] = value;
	array[2] = value;
}
