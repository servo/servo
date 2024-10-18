
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

// Function declaration.
ivec4 function(in ivec4 par);
bool is_all(const in ivec4 par, const in int value);
void set_all(out ivec4 par, const in int value);

void main (void)
{
	ivec4 par = ivec4(1, 1, 1, 1);
	ivec4 ret = ivec4(0, 0, 0, 0);

	float gray = 0.0;

	ret = function(par);

	// The parameter should remain unchanged by the function and the function should return 1.
	if(is_all(par, 1) && is_all(ret, 1))
	{
		gray = 1.0;
	}

	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}

// Function definition.
ivec4 function(in ivec4 par)
{
	// Return the value of the parameter.
	if(is_all(par, 1))
	{
		// Test parameter qualifier (default is "in").
		set_all(par, 0);

		return ivec4(1, 1, 1, 1);
	}
	else
		return ivec4(0, 0, 0, 0);
}

bool is_all(const in ivec4 par, const in int value)
{
	bool ret = true;

	if(par[0] != value)
		ret = false;
	if(par[1] != value)
		ret = false;
	if(par[2] != value)
		ret = false;
	if(par[3] != value)
		ret = false;

	return ret;
}

void set_all(out ivec4 par, const in int value)
{
	par[0] = value;
	par[1] = value;
	par[2] = value;
	par[3] = value;
}
