
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;
uniform float new_mad2[2];

void main (void)
{
	int i=0;
	float new_mad[2];
	float gray = 0.0;

	new_mad[0]=new_mad2[0];
	new_mad[1]=new_mad2[1];

	if( (new_mad[0] == 45.0) && (new_mad[1] == 14.0) )
	  gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
