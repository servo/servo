
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

bool _all(in bvec3 a)
{
	bool temp = true;

	if(!a[0]) temp = false;
	if(!a[1]) temp = false;
	if(!a[2]) temp = false;

	return temp;
}

void main (void)
{
	vec3 c = floor(4.0 * color.rgb);   // 3/4 true, 1/4 false
	gl_FragColor = vec4(vec3(_all(bvec3(c))), 1.0);
}
