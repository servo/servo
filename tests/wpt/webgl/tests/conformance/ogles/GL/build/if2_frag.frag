
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
    vec2 v;
    int i;
    if (v) // vectors cannot be used as conditional expression for if statement
        i++;
}
