
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

float gray = 0.0;

// Function declaration.
void function(void);

void main (void)
{
	gray = 0.0;

	function();

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

// Function definition.
void function(void)
{
	gray = 1.0;
}
