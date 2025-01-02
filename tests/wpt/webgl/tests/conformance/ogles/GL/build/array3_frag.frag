
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
    int array1[2], array2[2];
    bool b = array1 == array2; // equality operator does not work on arrays but works on array elements
}
