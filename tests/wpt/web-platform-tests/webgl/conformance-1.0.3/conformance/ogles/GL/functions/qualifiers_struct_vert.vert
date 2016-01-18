
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


attribute vec4 gtf_Vertex;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying vec4 color;



struct sabcd
{
	float a;
	float b;
	float c;
	float d;
};



sabcd qualifiers(in sabcd a, out sabcd b, inout sabcd c, const in sabcd d,
sabcd e)
{
        sabcd one = sabcd(1.0, 1.0, 1.0, 1.0);
        
        b = a;

        c.a += d.a;
        c.b += d.b;
        c.c += d.c;
        c.d += d.d;

        a.a += one.a;
        a.b += one.b;
        a.c += one.c;
        a.d += one.d;

        return e;
}

void main (void)
{
	sabcd a = sabcd(1.0, 1.0, 1.0, 1.0);
	sabcd b = sabcd(2.0, 2.0, 2.0, 2.0);
	sabcd c = sabcd(3.0, 3.0, 3.0, 3.0);
	sabcd d = sabcd(4.0, 4.0, 4.0, 4.0);
	sabcd e = sabcd(1.0, 1.0, 1.0, 1.0);
	sabcd f = sabcd(0.0, 0.0, 0.0, 0.0);
	sabcd one = sabcd(1.0, 1.0, 1.0, 1.0);
	sabcd four = sabcd(4.0, 4.0, 4.0, 4.0);
	sabcd seven = sabcd(7.0, 7.0, 7.0, 7.0);
	float q = 0.0;
	float q2 = 0.0;

	f = qualifiers(a, b, c, d, e);

	if(a == one) q += 1.0;
	if(b == one) q += 2.0;
	if(c == seven) q += 4.0;
	if(d == four) q2 += 1.0;
	if(e == one) q2 += 2.0;
	if(f == one) q2 += 4.0;

	color = vec4(vec2(q / 7.0, q2 / 7.0), 1.0, 1.0);
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
