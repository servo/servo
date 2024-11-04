
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
	if(color.r > 0.75 || color.g > 0.75 || color.b > 0.75)
	{
		/* The background color is black by default.
		 * Setting the fragment color to it simulates a discarded fragment.
		 */
		gl_FragColor = vec4(0.0, 0.0, 0.0, 0.0);
	}
	else
	{
		gl_FragColor = color;
	}
}
