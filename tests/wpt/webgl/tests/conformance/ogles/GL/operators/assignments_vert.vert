
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
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
	color = vec4(gray, gray, gray, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
