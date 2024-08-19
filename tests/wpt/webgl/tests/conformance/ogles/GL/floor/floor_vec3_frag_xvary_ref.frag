
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

vec3 floor_ref(vec3 x)
{
	if(x[0] >= 0.0)
		x[0] = float(int(x[0]));
	else
		x[0] = float(int(x[0]) - 1);
	if(x[1] >= 0.0)
		x[1] = float(int(x[1]));
	else
		x[1] = float(int(x[1]) - 1);
	if(x[2] >= 0.0)
		x[2] = float(int(x[2]));
	else
		x[2] = float(int(x[2]) - 1);
	return x;
}

void main (void)
{
	vec3 c = 10.0 * 2.0 * (color.rgb - 0.5);
	gl_FragColor = vec4((floor_ref(c) + 10.0) / 20.0, 1.0);
}
