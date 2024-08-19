
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform float funi1;
uniform vec2 funi2;
uniform vec3 funi3;
uniform vec4 funi4;
varying vec4 color;

void main (void)
{
	vec4 temp = vec4(funi1, funi2[0] + funi2[1], funi3[0] + funi3[1] + funi3[2], funi4[0] + funi4[1] + funi4[2] + funi4[3]);
	gl_FragColor = temp + color;
}
