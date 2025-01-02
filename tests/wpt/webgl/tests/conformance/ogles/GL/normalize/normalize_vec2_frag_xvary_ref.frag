
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
	vec4 tmp_Color = color + vec4(0.25);
	gl_FragColor = vec4(tmp_Color.rg / length(tmp_Color.rg), 0.0, 1.0);
}
