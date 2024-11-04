
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

const int array_size = 2;

void main (void)
{
	const mat3 a = mat3(1.0, 2.0, 3.0,
		            4.0, 5.0, 6.0,
			    7.0, 8.0, 9.0);
	const mat3 b = mat3(10.0, 11.0, 12.0,
		            13.0, 14.0, 15.0,
			    16.0, 17.0, 18.0);
	mat3 array[array_size];
	float gray;

	array[0] = a;
	array[1] = b;

	if((array[0] == a) && (array[1] == b))
		gray = 1.0;
	else
		gray = 0.0;

	gl_FragColor = vec4(gray, gray, gray, 1.0);
}

