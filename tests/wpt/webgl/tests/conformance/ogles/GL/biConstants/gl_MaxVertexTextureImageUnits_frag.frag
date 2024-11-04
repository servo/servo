
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
	// This test verifies that gl_MaxVertexTextureImageUnits is set and that its
	// value is greater than or equal to the minimum value.
	if(gl_MaxVertexTextureImageUnits >= 0)
		gl_FragColor = vec4(1.0);
	else
		gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);
}
