
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform int uniformInt;

void function(out int i)
{
    i = 1;
}

void main()
{
    function(uniformInt);  // out and inout parameters cannot be uniform since uniforms cannot be modified
}

