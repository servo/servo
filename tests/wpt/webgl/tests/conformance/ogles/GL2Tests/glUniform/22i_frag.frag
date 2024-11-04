
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform ivec2 color[2];

void main (void)
{
	float r = float(color[0][0]);
	float g = float(color[0][1]);
	float b = float(color[1][0]);
	float a = float(color[1][1]);

	gl_FragColor = vec4 (r/256.0, g/256.0, b/256.0, a/256.0);
}