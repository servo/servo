
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
    float f[];
    float flt = f[5];
    float f[3];  // higher array index has already been used
}
