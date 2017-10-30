
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


struct s {
    float f;
} s1 = s(1.0);

struct s3 {
   int i;
} s3Inst;

struct s2 {
    float f;
    s3 s3Inst;
} s2Inst = s2(1.0, s3(1));

void main()
{
    vec3 i = vec3(5.0, 4.0, ivec2(2.0, 1.0));
    ivec4 v2 = ivec4(1.0);
    vec4 v4 = vec4(v2);
    bvec4 v5 = bvec4(v2);
    vec3 v6 = vec3(v5);
    vec3 v = vec3(2, 2.0, 1);
    vec3 v1 = vec3(1.2, v);

    mat3 m1 = mat3(v,v,v);
    mat2 m2 = mat2(v, v6.x);
    
    gl_Position = vec4(1.0);
}

