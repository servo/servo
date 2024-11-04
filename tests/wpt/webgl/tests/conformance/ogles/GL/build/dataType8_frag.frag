
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
    f = 1.0;  // varyings cannot be written to in a fragment shader, they can be written to in a vertex shader
}
