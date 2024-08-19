
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct s {
    int i;
} s1;

void main()
{
   s1.i++;
   s1++;  // structure cannot be incremented
}
