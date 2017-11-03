
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
struct s{
    float f;
    vec3  v;
} s1 ;
void main()
{
    vec4 v = vec4(float(vec2(1,2)), 5,6,7);  // 1, 5, 6, 7
    vec4 v1 = vec4(3, vec2(ivec2(1,2)), 4);  // 3, 1, 2, 4
    vec4 v2 = vec4(8, 9, vec4(ivec4(1,2,3,4))); // 8,9, 1,2
    vec2 v3 = vec2(v2);  // 8,9
    vec4 v4 = vec4(v3, v2.z, v2.w);  // 8,9,1,2

    const vec4 v5 = vec4(2.0, s(2.0, vec3(3,4,5)).v); // 2,3,4,5
    gl_FragColor = v5 + v + v1 + v4 ;  // 14, 18, 13, 18
}
