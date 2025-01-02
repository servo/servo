
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
	vec3 c = floor(1.5 * color.rgb);   // 1/3 true, 2/3 false
	gl_FragColor = vec4(vec3(any(bvec3(c))), 1.0);
}
