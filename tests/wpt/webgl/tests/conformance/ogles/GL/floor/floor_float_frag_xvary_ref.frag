
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

float floor_ref(float x)
{
	if(x >= 0.0)
		x = float(int(x));
	else
		x = float(int(x) - 1);
	return x;
}

void main (void)
{
	float c = 10.0 * 2.0 * (color.r - 0.5);
	gl_FragColor = vec4((floor_ref(c) + 10.0) / 20.0, 0.0, 0.0, 1.0);
}
