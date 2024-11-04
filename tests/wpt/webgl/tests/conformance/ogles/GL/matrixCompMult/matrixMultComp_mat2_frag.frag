
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
	mat2 m1 = mat2(color.rg, color.ba);
	mat2 m2 = mat2(1.0, 0.5, 0.5, 1.0);
	mat2 m3 = mat2(0.0);

	m3 = matrixCompMult(m1, m2);
	gl_FragColor = vec4(m3[0][0], m3[1][0], m3[0][1], m3[1][1]);
}
