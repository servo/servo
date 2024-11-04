
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform vec4 color;
uniform ivec4 icolor;
uniform bool flag;

void main (void)
{
	if(flag)
		gl_FragColor = vec4 (icolor[0], icolor[1], icolor[2], icolor[3]);
	else
		gl_FragColor = vec4 (color[0], color[1], color[2], color[3]);
}