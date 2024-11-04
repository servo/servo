
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 col;

void main (void)
{
	gl_FragColor = vec4 (col[0], col[1], col[2], col[3]);
}
