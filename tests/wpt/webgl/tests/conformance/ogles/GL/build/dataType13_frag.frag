
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
    int i = 1;
    float f = 1.2;
    float result = f * i; // auto promotion now allowed
}
