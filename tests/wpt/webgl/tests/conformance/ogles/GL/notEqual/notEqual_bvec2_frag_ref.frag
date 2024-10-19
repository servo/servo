
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

bvec2 ne(in bvec2 a, in bvec2 b)
{
	bvec2 result;
	if(a[0] != b[0]) result[0] = true;
	else result[0] = false;
	if(a[1] != b[1]) result[1] = true;
	else result[1] = false;
	return result;
}

void main (void)
{
	vec2 c = floor(1.5 * color.rg);   // 1/3 true, 2/3 false
	vec2 result = vec2(ne(bvec2(c), bvec2(true)));
	gl_FragColor = vec4(result, 0.0, 1.0);
}
