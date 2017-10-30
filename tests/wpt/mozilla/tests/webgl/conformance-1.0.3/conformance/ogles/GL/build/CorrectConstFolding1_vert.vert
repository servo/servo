
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/


void main()
{

   const struct s2 { 
       int i;
       vec3 v3; 
       bvec4 bv4;
   } s22  = s2(8, vec3(9, 10, 11), bvec4(true, false, true, false));

   struct s4 {
       int ii;
       vec4 v4;
      };

   const struct s1 {
      s2 ss;
      int i;
      float f;
      mat4 m;
 	s4 s44;
   } s11 = s1(s22, 2, 4.0, mat4(5), s4(6, vec4(7, 8, 9, 10))) ;

  const int field3 = s11.i * s11.ss.i;  // constant folding (int * int)
  const vec4 field4 = s11.s44.v4 * s11.s44.v4; // constant folding (vec4 * vec4)
 // 49, 64, 81, 100
  const vec4 v4 = vec4(s11.ss.v3.y, s11.m[3][3], field3, field4[2]);  // 10.0, 5.0, 16.0, 81.0 
  gl_Position = v4;
}
