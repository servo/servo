#version 300 es

/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

precision mediump float;
uniform mediump sampler3D s3D;
uniform mediump sampler2DArray s2DArray;
out vec4 fragColor;
void main()
{
  fragColor = texture(s3D, vec3(0.5, 0.5, 0.5)) +
      texture(s2DArray, vec3(0.5, 0.5, 0.5));
}
