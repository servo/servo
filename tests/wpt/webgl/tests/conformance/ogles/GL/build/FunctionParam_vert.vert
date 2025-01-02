
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


int y = 1;

int foo(int, int b[y])  // array size should be constant
{
    return 1;
}

void main()
{
    int a[1];

    gl_Position = vec4(1.0);
}
