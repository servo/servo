
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
	vec4 al = color;
	vec3 m = al.yxw;
	float z = al.z;
	vec4 a = vec4(m.y, m.x, z, m.z);
	gl_FragColor = a;
}
