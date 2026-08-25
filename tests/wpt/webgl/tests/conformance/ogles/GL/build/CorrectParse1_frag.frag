
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
