
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void function(const out int i)
{  // out parameters cannot be const
   i = 3;
}

void main()
{
    int i;
    function(i);
}


