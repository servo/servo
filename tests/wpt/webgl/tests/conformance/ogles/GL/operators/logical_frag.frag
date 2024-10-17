
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

void main (void)
{
bool result = true;
	bool a = true;
	bool b = true;

	if( (a&&b) )
		result = result && true;
	else
		result = result && false;

	if( (a||b) )
		result = result && true;
	else
		result = result && false;

	if( !(a^^b) )
		result = result && true;
	else
		result = result && false;

	a = true;
	b = false;

	if( !(a&&b) )
		result = result && true;
	else
		result = result && false;

	if( (a||b) )
		result = result && true;
	else
		result = result && false;

	if( (a^^b) )
		result = result && true;
	else
		result = result && false;

	a = false;
	b = true;

	if( !(a&&b) )
		result = result && true;
	else
		result = result && false;

	if( (a||b) )
		result = result && true;
	else
		result = result && false;

	if( (a^^b) )
		result = result && true;
	else
		result = result && false;

	a = false;
	b = false;

	if( !(a&&b) )
		result = result && true;
	else
		result = result && false;

	if( !(a||b) )
		result = result && true;
	else
		result = result && false;

	if( !(a^^b) )
		result = result && true;
	else
		result = result && false;

	float gray;
	if( result )
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
