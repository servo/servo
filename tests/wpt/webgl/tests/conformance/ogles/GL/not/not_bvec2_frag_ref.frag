
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif
varying vec4 color;

bvec2 _not(in bvec2 a)
{
	bvec2 result;
	if(a[0]) result[0] = false;
	else result[0] = true;
	if(a[1]) result[1] = false;
	else result[1] = true;
	return result;
}

void main (void)
{
	vec2 c = floor(1.5 * color.rg);   // 1/3 true, 2/3 false
	gl_FragColor = vec4(vec2(_not(bvec2(c))), 0.0, 1.0);
}
