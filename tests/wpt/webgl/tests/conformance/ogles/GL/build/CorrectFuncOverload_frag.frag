
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void testVoid (vec4 v, vec4 v1)
{
}

void testVoid (ivec4 v, ivec4 v1)
{
}

void main(void)
{
    vec4 v;
    ivec4 i;
    testVoid(i, i);
    testVoid(v, v);
    gl_FragColor = v;
}
