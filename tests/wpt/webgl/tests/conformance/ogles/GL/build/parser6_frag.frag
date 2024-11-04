
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
    float f1,f2,f3;
    f3 = f1 > f2;  // f1 > f2 result in a bool that cannot be assigned to a float
}
