
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void main(void)
{
	if(gl_FrontFacing)
		gl_FragColor = vec4(0.0, 1.0, 0.0, 1.0);
	else
		gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
}
