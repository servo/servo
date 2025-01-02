
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
    float f1,f2;
    int i;
    bool b;
    float f3 = b ? i : f2; // second and third expression should of the type float
}
