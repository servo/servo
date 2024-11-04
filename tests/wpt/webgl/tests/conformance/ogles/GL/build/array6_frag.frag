
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
    const float index = 3.0;
    float f[index];  // arrays should be declared with an integer expression not float
}
