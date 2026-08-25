
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Color;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	vec3 x = 31.0 * gtf_Color.rgb + 1.0;
	vec3 y = vec3(0.0);
	vec3 z;   // x-1 / x+1
	int n = 50;

	// ln(x) = 2[x-1 + 1 (x-1)^3 + 1 (x-1)^5 + ...] for x > 0
	//          [x+1   3 (x+1)     5 (x+1)        ]
	z = (x - 1.0) / (x + 1.0);
	vec3 p = z;
	for(int i = 1; i <= 101; i += 2)
	{
		y += p / float(i);
		p *= z * z;
	}
	y *= 2.0;

	color = vec4(y / 3.466, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
