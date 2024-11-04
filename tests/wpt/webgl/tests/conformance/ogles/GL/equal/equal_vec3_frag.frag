
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
	vec3 c = floor(10.0 * color.rgb - 4.5);   // round to the nearest integer
	vec3 result = vec3(equal(c, vec3(0.0)));
	gl_FragColor = vec4(result, 1.0);
}
