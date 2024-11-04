
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
	vec3 c = 2.0 * (gtf_Color.rgb - 0.5);
	vec3 asin_c = vec3(0.0);
	vec3 scale = vec3(1.0);
	vec3 sign = vec3(1.0);

	// pow can't handle negative numbers so take advantage of symmetry
	if(c.r < 0.0)
	{
		sign.r = -1.0;
		c.r *= -1.0;
	}

	// Taylors series expansion for asin
	// 1000/2 iterations necessary to get the accuracy with this method
	for(int i = 1; i < 1000; i += 2)
	{
		asin_c.r += scale.r * pow(c.r, float(i)) / float(i);
		scale.r *= float(i) / float(i + 1);
	}

	// pow can't handle negative numbers so take advantage of symmetry
	if(c.g < 0.0)
	{
		sign.g = -1.0;
		c.g *= -1.0;
	}

	// Taylors series expansion for asin
	// 1000/2 iterations necessary to get the accuracy with this method
	for(int i = 1; i < 1000; i += 2)
	{
		asin_c.g += scale.g * pow(c.g, float(i)) / float(i);
		scale.g *= float(i) / float(i + 1);
	}

	// pow can't handle negative numbers so take advantage of symmetry
	if(c.b < 0.0)
	{
		sign.b = -1.0;
		c.b *= -1.0;
	}

	// Taylors series expansion for asin
	// 1000/2 iterations necessary to get the accuracy with this method
	for(int i = 1; i < 1000; i += 2)
	{
		asin_c.b += scale.b * pow(c.b, float(i)) / float(i);
		scale.b *= float(i) / float(i + 1);
	}

	color = vec4(sign * asin_c / M_PI + 0.5, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
