
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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
