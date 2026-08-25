
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct s {
    int i = 1.0;  // struct members cannot be initialized at the time of structure declaration
} s1;

void main()
{
}
