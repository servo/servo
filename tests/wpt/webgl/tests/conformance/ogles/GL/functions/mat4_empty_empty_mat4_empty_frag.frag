
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
mat4 function(mat4 par);
bool is_all(const in mat4 par, const in float value);
void set_all(out mat4 par, const in float value);

void main (void)
{
	mat4 par = mat4(1.0, 1.0, 1.0, 1.0,
			1.0, 1.0, 1.0, 1.0,
			1.0, 1.0, 1.0, 1.0,
			1.0, 1.0, 1.0, 1.0);
	mat4 ret = mat4(0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0,
			0.0, 0.0, 0.0, 0.0);

	float gray = 0.0;

	ret = function(par);

	// The parameter should remain unchanged by the function and the function should return 1.0.
	if(is_all(par, 1.0) && is_all(ret, 1.0))
	{
		gray = 1.0;
	}

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
mat4 function(mat4 par)
{
	// Return the value of the parameter.
	if(is_all(par, 1.0))
	{
		// Test parameter qualifier (default is "in").
		set_all(par, 0.0);

		return mat4(1.0, 1.0, 1.0, 1.0,
			    1.0, 1.0, 1.0, 1.0,
			    1.0, 1.0, 1.0, 1.0,
			    1.0, 1.0, 1.0, 1.0);
	}
	else
		return mat4(0.0, 0.0, 0.0, 0.0,
			    0.0, 0.0, 0.0, 0.0,
			    0.0, 0.0, 0.0, 0.0,
			    0.0, 0.0, 0.0, 0.0);
}

bool is_all(const in mat4 par, const in float value)
{
	bool ret = true;

	if(par[0][0] != value)
		ret = false;
	if(par[0][1] != value)
		ret = false;
	if(par[0][2] != value)
		ret = false;
	if(par[0][3] != value)
		ret = false;

	if(par[1][0] != value)
		ret = false;
	if(par[1][1] != value)
		ret = false;
	if(par[1][2] != value)
		ret = false;
	if(par[1][3] != value)
		ret = false;

	if(par[2][0] != value)
		ret = false;
	if(par[2][1] != value)
		ret = false;
	if(par[2][2] != value)
		ret = false;
	if(par[2][3] != value)
		ret = false;

	if(par[3][0] != value)
		ret = false;
	if(par[3][1] != value)
		ret = false;
	if(par[3][2] != value)
		ret = false;
	if(par[3][3] != value)
		ret = false;

	return ret;
}

void set_all(out mat4 par, const in float value)
{
	par[0][0] = value;
	par[0][1] = value;
	par[0][2] = value;
	par[0][3] = value;

	par[1][0] = value;
	par[1][1] = value;
	par[1][2] = value;
	par[1][3] = value;

	par[2][0] = value;
	par[2][1] = value;
	par[2][2] = value;
	par[2][3] = value;

	par[3][0] = value;
	par[3][1] = value;
	par[3][2] = value;
	par[3][3] = value;
}
