
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void main()
{
    vec4 v;
    vec4 v1 = (vec4) v; // incorrect typecasting, vec4(v) is correct
}
