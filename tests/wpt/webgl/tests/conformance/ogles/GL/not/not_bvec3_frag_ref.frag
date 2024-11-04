
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

bvec3 _not(in bvec3 a)
{
	bvec3 result;
	if(a[0]) result[0] = false;
	else result[0] = true;
	if(a[1]) result[1] = false;
	else result[1] = true;
	if(a[2]) result[2] = false;
	else result[2] = true;
	return result;
}

void main (void)
{
	vec3 c = floor(1.5 * color.rgb);   // 1/3 true, 2/3 false
	gl_FragColor = vec4(vec3(_not(bvec3(c))), 1.0);
}
