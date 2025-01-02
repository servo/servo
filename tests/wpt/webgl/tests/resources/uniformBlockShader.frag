#version 300 es

/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

precision mediump float;

in vec3 normal;
in vec4 ecPosition;

out vec4 fragColor;

void main()
{
    fragColor = vec4(normal/2.0+vec3(0.5), 1);
}
