
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
bvec4 function(inout bvec4 par);
bool is_all(const in bvec4 par, const in bool value);
void set_all(out bvec4 par, const in bool value);

void main (void)
{
	bvec4 par = bvec4(true, true, true, true);
	bvec4 ret = bvec4(false, false, false, false);

	float gray = 0.0;

	ret = function(par);

	// The parameter should be changed by the function and the function should return true.
	if(is_all(par, false) && is_all(ret, true))
	{
		gray = 1.0;
	}

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
bvec4 function(inout bvec4 par)
{
	// Return the value of the parameter.
	if(is_all(par, true))
	{
		// Test parameter qualifier (default is "in").
		set_all(par, false);

		return bvec4(true, true, true, true);
	}
	else
		return bvec4(false, false, false, false);
}

bool is_all(const in bvec4 par, const in bool value)
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

void set_all(out bvec4 par, const in bool value)
{
	par[0] = value;
	par[1] = value;
	par[2] = value;
	par[3] = value;
}
