
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

bool _any(in bvec2 a)
{
	bool temp = false;

	if(a[0]) temp = true;
	if(a[1]) temp = true;

	return temp;
}

void main (void)
{
	vec2 c = floor(1.5 * color.rg);   // 1/3 true, 2/3 false
	gl_FragColor = vec4(vec3(_any(bvec2(c))), 1.0);
}
