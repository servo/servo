
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
	vec3 result = vec3(notEqual(bvec3(c), bvec3(true)));
	gl_FragColor = vec4(result, 1.0);
}
