
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
	const vec3 edge = vec3(0.5, 0.5, 0.5);
	vec3 c = color.rgb;
	if(c[0] >= edge[0])
	{
		c[0] = 1.0;
	}
	else
	{
		c[0] = 0.0;
	}
	if(c[1] >= edge[1])
	{
		c[1] = 1.0;
	}
	else
	{
		c[1] = 0.0;
	}
	if(c[2] >= edge[2])
	{
		c[2] = 1.0;
	}
	else
	{
		c[2] = 0.0;
	}

	gl_FragColor = vec4(c, 1.0);
}
