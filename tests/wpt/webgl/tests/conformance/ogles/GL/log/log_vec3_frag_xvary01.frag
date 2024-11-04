
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
	vec3 c = (color.rgb + 0.01) / 1.01;
	gl_FragColor = vec4(log(c) / -4.61, 1.0);
}
