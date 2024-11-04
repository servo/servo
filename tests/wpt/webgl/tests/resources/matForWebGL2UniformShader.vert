#version 300 es

/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

uniform mat2x3 mval2x3;
uniform mat2x4 mval2x4;
uniform mat3x2 mval3x2;
uniform mat3x4 mval3x4;
uniform mat4x2 mval4x2;
uniform mat4x3 mval4x3;

void main()
{

  gl_Position = vec4(mval2x3 * vec2(1.0, 2.0), 0.0) +
      mval2x4 * vec2(1.0, 2.0) +
      vec4(mval3x2 * vec3(1.0, 2.0, 3.0), 0.0, 0.0) +
      mval3x4 * vec3(1.0, 2.0, 3.0) +
      vec4(mval4x2 * vec4(1.0, 2.0, 3.0, 4.0), 0.0, 0.0) +
      vec4(mval4x3 * vec4(1.0, 2.0, 3.0, 4.0), 0.0);
}
