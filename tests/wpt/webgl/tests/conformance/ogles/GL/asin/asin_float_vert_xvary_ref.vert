
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Vertex;
attribute vec4 gtf_Color;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;

void main (void)
{
	const float M_PI = 3.14159265358979323846;
	float c = 2.0 * (gtf_Color.r - 0.5);

	float asin_c = 0.0;
	float scale = 1.0;
	float sign = 1.0;

	// pow can't handle negative numbers so take advantage of symmetry
	if(c < 0.0)
	{
		sign = -1.0;
		c *= -1.0;
	}

	// Taylors series expansion for asin
	// 1000/2 iterations necessary to get the accuracy with this method
	for(int i = 1; i < 1000; i += 2)
	{
		asin_c += scale * pow(c, float(i)) / float(i);
		scale *= float(i) / float(i + 1);
	}

	color = vec4(sign * asin_c / M_PI + 0.5, 0.0, 0.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
