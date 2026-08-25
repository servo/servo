
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
	vec3 c = 100.0 * color.rgb;
	gl_FragColor = vec4(c / 100.0, 1.0);
}
