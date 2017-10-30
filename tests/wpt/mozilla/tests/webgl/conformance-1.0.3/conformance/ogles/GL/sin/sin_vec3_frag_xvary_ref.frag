
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

float lerp(float a, float b, float s)
{
	return a + (b - a) * s;
}

void main (void)
{
	float sinValues[17];
	sinValues[0] = 0.0;
	sinValues[1] = 0.382683;
	sinValues[2] = 0.707107;
	sinValues[3] = 0.92388;
	sinValues[4] = 1.0;
	sinValues[5] = 0.92388;
	sinValues[6] = 0.707107;
	sinValues[7] = 0.382683;
	sinValues[8] = 0.0;
	sinValues[9] = -0.382683;
	sinValues[10] = -0.707107;
	sinValues[11] = -0.92388;
	sinValues[12] = -1.0;
	sinValues[13] = -0.923879;
	sinValues[14] = -0.707107;
	sinValues[15] = -0.382683;
	sinValues[16] = 0.0;

	const float M_PI = 3.14159265358979323846;
	vec3 c = 2.0 * M_PI * color.rgb;
	
	vec3 arrVal = c * 2.546478971;
	int arr0x = int(floor(arrVal.x));
	int arr0y = int(floor(arrVal.y));
	int arr0z = int(floor(arrVal.z));
	vec3 weight = arrVal - floor(arrVal);
	vec3 sin_c = vec3(0.0, 0.0, 0.0);

	if (arr0x == 0)
		sin_c.x = lerp(sinValues[0], sinValues[1], weight.x);
	else if (arr0x == 1)
		sin_c.x = lerp(sinValues[1], sinValues[2], weight.x);
	else if (arr0x == 2)
		sin_c.x = lerp(sinValues[2], sinValues[3], weight.x);
	else if (arr0x == 3)
		sin_c.x = lerp(sinValues[3], sinValues[4], weight.x);
	else if (arr0x == 4)
		sin_c.x = lerp(sinValues[4], sinValues[5], weight.x);
	else if (arr0x == 5)
		sin_c.x = lerp(sinValues[5], sinValues[6], weight.x);
	else if (arr0x == 6)
		sin_c.x = lerp(sinValues[6], sinValues[7], weight.x);
	else if (arr0x == 7)
		sin_c.x = lerp(sinValues[7], sinValues[8], weight.x);
	else if (arr0x == 8)
		sin_c.x = lerp(sinValues[8], sinValues[9], weight.x);
	else if (arr0x == 9)
		sin_c.x = lerp(sinValues[9], sinValues[10], weight.x);
	else if (arr0x == 10)
		sin_c.x = lerp(sinValues[10], sinValues[11], weight.x);
	else if (arr0x == 11)
		sin_c.x = lerp(sinValues[11], sinValues[12], weight.x);
	else if (arr0x == 12)
		sin_c.x = lerp(sinValues[12], sinValues[13], weight.x);
	else if (arr0x == 13)
		sin_c.x = lerp(sinValues[13], sinValues[14], weight.x);
	else if (arr0x == 14)
		sin_c.x = lerp(sinValues[14], sinValues[15], weight.x);
	else if (arr0x == 15)
		sin_c.x = lerp(sinValues[15], sinValues[16], weight.x);
        else if (arr0x == 16)
                sin_c.x = sinValues[16];
		
	if (arr0y == 0)
		sin_c.y = lerp(sinValues[0], sinValues[1], weight.y);
	else if (arr0y == 1)
		sin_c.y = lerp(sinValues[1], sinValues[2], weight.y);
	else if (arr0y == 2)
		sin_c.y = lerp(sinValues[2], sinValues[3], weight.y);
	else if (arr0y == 3)
		sin_c.y = lerp(sinValues[3], sinValues[4], weight.y);
	else if (arr0y == 4)
		sin_c.y = lerp(sinValues[4], sinValues[5], weight.y);
	else if (arr0y == 5)
		sin_c.y = lerp(sinValues[5], sinValues[6], weight.y);
	else if (arr0y == 6)
		sin_c.y = lerp(sinValues[6], sinValues[7], weight.y);
	else if (arr0y == 7)
		sin_c.y = lerp(sinValues[7], sinValues[8], weight.y);
	else if (arr0y == 8)
		sin_c.y = lerp(sinValues[8], sinValues[9], weight.y);
	else if (arr0y == 9)
		sin_c.y = lerp(sinValues[9], sinValues[10], weight.y);
	else if (arr0y == 10)
		sin_c.y = lerp(sinValues[10], sinValues[11], weight.y);
	else if (arr0y == 11)
		sin_c.y = lerp(sinValues[11], sinValues[12], weight.y);
	else if (arr0y == 12)
		sin_c.y = lerp(sinValues[12], sinValues[13], weight.y);
	else if (arr0y == 13)
		sin_c.y = lerp(sinValues[13], sinValues[14], weight.y);
	else if (arr0y == 14)
		sin_c.y = lerp(sinValues[14], sinValues[15], weight.y);
	else if (arr0y == 15)
		sin_c.y = lerp(sinValues[15], sinValues[16], weight.y);
        else if (arr0y == 16)
                sin_c.y = sinValues[16];
		
	if (arr0z == 0)
		sin_c.z = lerp(sinValues[0], sinValues[1], weight.z);
	else if (arr0z == 1)
		sin_c.z = lerp(sinValues[1], sinValues[2], weight.z);
	else if (arr0z == 2)
		sin_c.z = lerp(sinValues[2], sinValues[3], weight.z);
	else if (arr0z == 3)
		sin_c.z = lerp(sinValues[3], sinValues[4], weight.z);
	else if (arr0z == 4)
		sin_c.z = lerp(sinValues[4], sinValues[5], weight.z);
	else if (arr0z == 5)
		sin_c.z = lerp(sinValues[5], sinValues[6], weight.z);
	else if (arr0z == 6)
		sin_c.z = lerp(sinValues[6], sinValues[7], weight.z);
	else if (arr0z == 7)
		sin_c.z = lerp(sinValues[7], sinValues[8], weight.z);
	else if (arr0z == 8)
		sin_c.z = lerp(sinValues[8], sinValues[9], weight.z);
	else if (arr0z == 9)
		sin_c.z = lerp(sinValues[9], sinValues[10], weight.z);
	else if (arr0z == 10)
		sin_c.z = lerp(sinValues[10], sinValues[11], weight.z);
	else if (arr0z == 11)
		sin_c.z = lerp(sinValues[11], sinValues[12], weight.z);
	else if (arr0z == 12)
		sin_c.z = lerp(sinValues[12], sinValues[13], weight.z);
	else if (arr0z == 13)
		sin_c.z = lerp(sinValues[13], sinValues[14], weight.z);
	else if (arr0z == 14)
		sin_c.z = lerp(sinValues[14], sinValues[15], weight.z);
	else if (arr0z == 15)
		sin_c.z = lerp(sinValues[15], sinValues[16], weight.z);
        else if (arr0z == 16)
                sin_c.z = sinValues[16];

	gl_FragColor = vec4(0.5 * sin_c + 0.5, 1.0);
}
