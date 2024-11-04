
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying float f;
void main()
{
    float flt = 1.0;
    flt++;
    f++;  // varyings in a fragment shader are read only
}
