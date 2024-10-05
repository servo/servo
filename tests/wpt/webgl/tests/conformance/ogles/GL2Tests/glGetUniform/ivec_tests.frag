
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform int funi1;
uniform ivec2 funi2;
uniform ivec3 funi3;
uniform ivec4 funi4;
varying vec4 color;

void main (void)
{
	vec4 temp = vec4(float(funi1), float(funi2[0] + funi2[1]), float(funi3[0] + funi3[1] + funi3[2]), float(funi4[0] + funi4[1] + funi4[2] + funi4[3]));
	gl_FragColor = temp + color;
}
