#version 300 es

/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

uniform uint uval;
uniform uvec2 uval2;
uniform uvec3 uval3;
uniform uvec4 uval4;

void main()
{
    uint sum = uval
            + uval2[0] + uval2[1]
            + uval3[0] + uval3[1] + uval3[2]
            + uval4[0] + uval4[1] + uval4[2] + uval4[3];
    gl_Position = vec4(sum, 0.0, 0.0, 1.0);
}
