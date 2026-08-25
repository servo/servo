
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void function(inout int i);

void main()
{
    int i;
    function(i);
}

// function definition has different parameter qualifiers than function declaration
void function(in int i)
{
   i = 3;
}
