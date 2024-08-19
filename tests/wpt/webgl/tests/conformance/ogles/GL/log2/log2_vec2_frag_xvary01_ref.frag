
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;
const float ln2 = 0.69314718055994530941723212145818;



void main (void)
{
	vec2 x = (color.rg + 0.01) / 1.01;
	vec2 y = vec2(0.0);
	vec2 z;   // x-1 / x+1
	int n = 50;

	// ln(x) = 2[x-1 + 1 (x-1)^3 + 1 (x-1)^5 + ...] for x > 0
	//          [x+1   3 (x+1)     5 (x+1)        ]
	// Note: z will always be negative between 0.01 and 1.0 and
	// so will y since it is raised to an odd power, and the shader spec
	// does not support pow(-x, y) where y is not a compile time constant
	z = abs((x - 1.0) / (x + 1.0));
	vec2 p = z;
	for(int i = 1; i <= 101; i += 2)
	{
		y += p / float(i);
		p *= z * z;
	}
	y *= -2.0 / ln2;

	gl_FragColor = vec4(y / -8.0, 0.0, 1.0);
}
