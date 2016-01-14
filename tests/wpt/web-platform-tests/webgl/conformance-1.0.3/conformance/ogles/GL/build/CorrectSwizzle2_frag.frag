
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
void main()
{
    float f, f1, f2;
    f = f1 = f2;
    f += f1 += f2;
    
    vec4 v, v1, v2;
    v = v1 = v2;
    v += v1 += v2;
    v.wx = v1.zx = v2.yx;
    v.wx += v1.zx += v2.yx;

    mat4  m, m1, m2;
    m = m1 = m2;
    m += m1 += m2;
    m[3].wx = m1[2].zx = m2[1].yx;
    m[3].wx += m1[2].zx += m2[1].yx;

    mat4  am[4], am1[4], am2[4];
    am[3] = am1[2] = am2[1];
    am[3] += am1[2] += am2[1];
    am[3][3].wx = am1[2][2].zx = am2[1][1].yx;
    am[3][3].wx += am1[2][2].zx += am2[1][1].yx;
    am[3][3].wx += am1[2][2].zx += ++(am2[1][1].yx);
    am[3][3].wx += am1[2][2].zx += (am2[1][1].yx)++;

    gl_FragColor = vec4(am[3][3].z, m[3].w, v.w, f);
}
