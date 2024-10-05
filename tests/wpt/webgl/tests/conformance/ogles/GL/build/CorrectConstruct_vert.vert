
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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

