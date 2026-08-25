
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
	const float exp1 = 2.7183;
	const float exp3 = 20.0855;
	vec3 c = color.rgb;
	gl_FragColor = vec4(pow(vec3(exp1), 3.0 * c) / exp3, 1.0);
}
