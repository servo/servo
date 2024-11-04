
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
	int m = 12;
	int n = 102;
	bool result = true;
	int r = m;

	if( r==12 )
		result = result && true;
	else
		result = result && false;

	r += m;

	if( r == 24 )
		result = result && true;
	else
		result = result && false;

	r-= m;

	if( r == 12 )
		result = result && true;
	else
		result = result && false;

	r*= m;

	if ( r == 144 )
		result = result && true;
	else
		result = result && false;

	r/= m;

	// Integer divide can be implemented via float reciprocal,
	// so the result need not be exact
	if( r >= 11 && r <= 13 )
		result = result && true;
	else
		result = result && false;

	float gray;
	if( result )
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
