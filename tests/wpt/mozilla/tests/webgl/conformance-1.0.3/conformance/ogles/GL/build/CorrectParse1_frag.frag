
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
uniform vec3 a[8];

uniform bool ub;
varying mat4 vm;

int foo(float);

float bar(int i)
{
    return float(i);
}

void main (void)
{
    const int x = 3;
    mat4 a[4]; 
    vec4 v;

    for (float f = 0.0; f != 3.0; ++f)
    {
    }

    vec3 v3[x + x];

    int vi = foo(2.3);

    vec3 v3_1 = v3[x];

    float f1 = a[x][2].z * float(x);  
    f1 = a[x][2][2] * float(x);
    f1 = v[2] * v[1];

    const int ci = 2;

}

int foo(float f)
{
    return 2;
}
