
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
	ivec4 init = ivec4(2,3,5,9);
	vec4 a = vec4(init);
	float gray;
	if( (a[0] == 2.0) && (a[1] == 3.0) && (a[2] == 5.0) && (a[3] == 9.0) )
	gray=1.0;
	else gray=0.0;
	gl_FragColor = vec4(gray, gray, gray, 1.0);
}
