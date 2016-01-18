
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
    struct s5 {
    float k;
    };
    const struct s {
        int i;
    	float j;
      s5 s55;
    } ss = s(4,1.0, s5(1.0));


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


   const struct s7 {
       int i;
       mat3 m3;
   } s77 = s7(12, mat3(15));

  vec2       v21 = vec2(1);  // Not a constant 
  const vec2 v22 = vec2(11); // 11.0, 11.0
  const vec4 v41 = vec4(2);  // 2.0, 2.0, 2.0, 2.0
  const vec4 v43 = vec4(4,4,4,4); // 4.0, 4.0, 4.0, 4.0
  const vec4 v44 = vec4(5.0, 5.0, 5.0, 5.0); // 5.0, 5.0, 5.0, 5.0
  const vec4 v45 = vec4(v22, v22);  // 11.0, 11.0, 11.0, 11.0
  const vec4 v46 = vec4(vec2(20, 21), vec2(22, 23));  // 20.0, 21.0, 22.0, 23.0

  const vec3 v31 = vec3(s22.v3);  // 9.0, 10.0, 11.0
  const vec3 v32 = vec3(s77.m3);  // 15.0, 0, 0 
  const vec3 v33 = vec3(s77.m3[2]); // 0, 0, 15.0
  const vec3 v34 = vec3(s77.m3[2][0]);  // 0,0,0

  
  const mat4 m41 = mat4(1);  // 1,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,1
  const mat4 m42 = mat4(v44, v44, v44, v44);  // all 5s
  const mat4 m43 = mat4( v43.x);  // 4,0,0,0,0,4,0,0,0,0,0,4,0,0,0,0,0,4

  const vec4 v47 = vec4(m41[0][0]);  // 1.0,1.0,1.0,1.0

  const mat4 m45 = mat4(s22.v3, v44, v45, v32, 50, 52);  //9,10,11,5,5,5,5,11,11,11,11,15.0, 0,0, 50.0, 52.0 
  //const mat3 m31 = mat3(1, mat2(1), 2.0, vec3(1));  // 1.0, 1,0,0,1,2,1,1,1
  const vec4 v48 = vec4(v31[0], v22[1], v41[0], v43[3]);  //9, 11, 2, 4
  const vec4 v49 = vec4(s22.v3.xy, s22.v3.zx); // 9,10,11,9
  const vec4 v410 = vec4(v44.xy, v43.zx);  //5,5,4,4

  const vec4 v411 = vec4(m42[3]);  // 5,5,5,5
  const vec4 v412 = vec4(m43[2]);  // 0,0,4,0

  const vec2 v23 = vec2(m41);  // 1,0
  
  const vec2 v24 = vec2(33, s11.i);  // 33, 2

  const vec4 v413 = vec4(vec2(1.0,2.0),ivec2(3.0,4.0));  // 1,2,3,4 
  const ivec4 i41 = ivec4(1.0, 2.0, 3.0, 4.0);  // 1,2,3,4
  
  const ivec4 i42 = ivec4(6);  // 6,6,6,6
  const ivec4 i43 = ivec4(v45);  //11,11,11,11

  const ivec4 i44 = ivec4(v44[0]);  // 5,5,5,5
  const ivec4 i45 = ivec4(vec2(20, 21), vec2(22, 23));  // 20, 21, 22, 23
  const vec4 v414 = vec4(ivec2(29, 30), ivec2(31, 32)); // 29.0, 30.0, 31.0, 32.0 
  const ivec4 i46 = ivec4(ivec2(2.0,3.0), ivec3(4.0,5.0,6.0));
  const ivec4 i47 = ivec4(i46);  // 2,3,4,5
  const ivec4 i48 = ivec4(v414.x);  // 29,29,29,29

  const ivec4 i49 = ivec4(vec4(1)); // 1,1,1,1
  const ivec4 i414 = ivec4(mat4(14)); // 14, 0,0,0,
  const ivec4 i410 = ivec4(m43);  // 4,0,0,0
  const ivec4 i411 = ivec4(m43[1]);  // 0, 4, 0, 0
  const ivec4 i412 = ivec4(s77.i); // 12, 12, 12, 12
  const ivec4 i416 = ivec4(s22.v3.zyx, 12);  // 11, 10, 9, 12

  const vec4 v415 = vec4(ivec2(35), ivec2(36)); // 35.0, 35.0 ,36.0 , 36.0 

  const bvec4 b41 = bvec4(1.0, 2.0, 3.0, 4.0);  // true,true,true,true
  
  const bvec4 b42 = bvec4(6);  // true,true,true,true
  const bvec4 b43 = bvec4(v45);  //true,true,true,true

  const bvec4 b44 = bvec4(v44[0]);  // true,true,true,true
  const bvec4 b45 = bvec4(vec2(0, 21), vec2(0, 1));  // false, true, false, true
  const bvec4 b46 = bvec4(ivec2(0.0,3.0), ivec3(0,5.0,6.0)); // false, true, false, true
  const bvec4 b47 = bvec4(i46);  // true,true,true,true
  const bvec4 b48 = bvec4(v414.x);  // true,true,true,true

  const bvec4 b49 = bvec4(vec4(0)); // false,false,false,false
  const bvec4 b414 = bvec4(mat4(14)); // true, false,false,false,
  const bvec4 b410 = bvec4(m43);  // true,false,false,false
  const bvec4 b411 = bvec4(m43[1]);  // false, true, false, false
  const bvec4 b412 = bvec4(s77.i) ; // true, true, true, true

  const vec3 v35 = vec3(s11.s44.v4);  // 7.0,8.0,9.0


  struct s10 {
     int k;
  };
  struct s9 {
       float f;
      s10 s101;
   }; 
  const struct s8 {
      int i;
      s9 s99;
  } s88 = s8(1, s9(2.0, s10(5)));

   struct st4 {
       int m;
       vec3 v3;
   };
   struct st3 {
      int k;
      int l;
      st4 st44;
     };
   struct st2 {
       float f;
       st3 st33;
  }; 
  const struct st1 {
      int i;
      st2 st22;
  } st11 = st1(1, st2(2.0, st3(5, 6, st4(7, v35))));

  const vec4 v416 = vec4(s88.s99.s101.k); // all 5s
  const vec4 v417 = vec4(st11.st22.st33.st44.v3, s88.s99.s101.k);  // 7.0, 8.0, 9.0, 5.0
  const vec3 v36 = vec3(s11.ss.v3);  // 9, 10, 11

  vec4 v418 = v416;  // all 5s
  const float f1 = v416[0];  // 5.0
  vec4 v419;
  v419.xyz = st11.st22.st33.st44.v3;
  mat4 m47;

  struct struct2 {
      int k;
  } struct22 = struct2(4);

  const struct struct1 {
       struct2 sst2;
  } struct11 = struct1(struct2(2));

  const vec4 v420 = v417;  // 7.0, 8.0, 9.0 , 5.0
  
  vec4 v421 = vec4(s11.m);  // 5, 0, 0, 0
  vec4 v422 = v420;  // 7.0, 8.0, 9.0 , 5.0

  vec4 v423 = s11.s44.v4;   // 7, 8, 9, 10
  
  int int1 = ss.i * ss.i;  // 16
  int int2 = ss.i * 2;  // 8

  const vec4 v425 = v420 * v420;  // 49, 64, 81, 25
  const vec4 v426 = s11.m * s11.s44.v4; // 35, 40, 45, 50
  const vec4 v427 = s11.s44.v4 * s11.m; // 35, 40, 45, 50
  
  float ff = 2.0; 
  const float ffConst = 2.0;
  
  vec4 v428 = ff + v425;  // ordinary assignment with binary node
  vec3 v39 = vec3(5);

  vec3 v310 = s22.v3 + v39;  //14, 15, 16

  const vec4 v429 = v420 + v420; // 14, 16, 18, 10
  const vec4 v430 = v420 + ffConst;  // 9, 10, 11,7 
  const vec4 v432 =  v429 + s11.f;  // 18, 20, 22, 14

  const vec4 v433 = vec4(s11.f + s11.f);  // all 8s
  const vec4 v434 = v432 + vec4(3);  // 21, 23, 25, 17
  const mat4 m48 = s11.m + ffConst;  // diagonal 7s and others 2s
  const mat4 m49 = mat4(ffConst + s11.f);  // diagonal 6s
  const mat4 m410 = m48 + s11.f;  // diagonal 11, others - 6s

  const mat4 m413 = m48 + m48 ; // diagonal 14, others 4
  const mat4 m414 = m413 + ffConst ; // diagonal 16, others 6 

  const vec4 v435 = ffConst + v420;  // 9, 10, 11,7 
  const vec4 v436 =  s11.f + v429;  // 18, 20, 22, 14
  const mat4 m415 = ffConst + s11.m;  // diagonal 7s and others 2s
  const mat4 m416 = s11.f + m48 ;  // diagonal 11, others - 6s
  const mat4 m417 = ffConst + m413 ; // diagonal 16, others 6 

  const vec4 v437 = v420 - v420; // 0, 0, 0, 0
  const vec4 v438 = v420 - ffConst;  // 5, 6, 7,3 
  const vec4 v440 =  v429 - s11.f;  // 10, 12, 14, 6

  const vec4 v441 = vec4(s11.f - s11.f);  // all 0s
  const vec4 v442 = v432 - vec4(3);  // 15, 17, 19, 11
  const mat4 m418 = s11.m - ffConst;  // diagonal 3s and others -2s
  const mat4 m419 = mat4(ffConst - s11.f);  // diagonal -> -2s
  const mat4 m420 = m48 - s11.f;  // diagonal 3, others -> -2

  const mat4 m423 = m48 - m48 ; // All 0s
  const mat4 m424 = m413 - ffConst ; // diagonal 12, others 2 

  const vec4 v443 = ffConst - v420;  // -5, -6, -7,-3 
  const vec4 v444 =  s11.f - v429;  // -10, -12, -14, -6
  const mat4 m425 = ffConst - s11.m;  // diagonal -3s and others 2s
  const mat4 m426 = s11.f - m48 ;  // diagonal -3, others  2s
  const mat4 m427 = ffConst - m413 ; // diagonal -12, others -2 

  const vec4 v445 = v420 * v420; // 49, 64, 81, 25
  const vec4 v446 = v420 * ffConst;  // 14, 16, 18,10 
  const vec4 v448 =  v429 * s11.f;  // 56, 46, 72, 40

  const vec4 v449 = vec4(s11.f * s11.f);  // all 16
  const vec4 v450 = v432 * vec4(3);  // 54, 60, 66, 42
  const mat4 m428 = s11.m * ffConst;  // diagonal 10 and others 0s
  const mat4 m429 = mat4(ffConst * s11.f);  // diagonal 8
  const mat4 m430 = m48 * s11.f;  // diagonal 28, others 8

  const mat4 m433 = m48 * m48 ; // diagonal 61, others 36
  const mat4 m434 = m413 * ffConst ; // diagonal 28, others 8 

  const vec4 v451 = ffConst * v420;  // 14, 16, 18,10 
  const vec4 v452 =  s11.f * v429;  // 56, 64, 72, 40
  const mat4 m435 = ffConst * s11.m;  //  diagonal 10 and others 0s
  const mat4 m436 = s11.f * m48 ;  // diagonal 28, others - 8s
  const mat4 m437 = ffConst * m413 ; // diagonal 28, others 8

  const vec4 v453 = v420 / v420; // 1, 1, 1, 1
  const vec4 v454 = v420 / ffConst;  // 3.5, 4, 4.5,2.5 

  const vec4 v457 = vec4(s11.f / s11.f);  // all 1s
  const vec4 v458 = v432 / vec4(3);  // 6, 6.6666, 7.333, 4.6666
  const mat4 m438 = s11.m / ffConst;  // diagonal 2.5 and others 0s
  const mat4 m439 = mat4(ffConst / s11.f);  // diagonal 0.5s
  const mat4 m440 = m48 / s11.f;  // diagonal 1.75, others 0.5s 

  const mat4 m443 = m48 / m48 ; // All 1s
  const mat4 m444 = m413 / ffConst ; // diagonal 7, others 2 

  const vec4 v459 = ffConst / v420;  // .2857 , .25, .22, .4
  const vec4 v460 =  s11.f / v429;  // .2857, .25, .22, .4
  //const mat4 m445 = ffConst / s11.m;  // divide by zero error
  const mat4 m446 = s11.f / m48 ;  // diagonal .571,  others 2
  const mat4 m447 = ffConst / m413 ; // diagonal .1428, others 0.5

  const vec4 v461 = v453 * m428; // 10, 10, 10, 10
  const vec4 v462 = v453 * m437; // 52, 52, 52, 52
  const vec4 v463 = m428 * v451; // 140, 160, 180, 100
  const vec4 v464 = m437 * v451; // 744, 784, 824, 664

  int ii = 2; 
  const int iiConst = 2;

  const ivec4 i420 = ivec4( 7,8,9,5);  // 7, 8, 9, 5

  const ivec4 i429 = i420 + i420; // 14, 16, 18, 10
  const ivec4 i430 = i420 + iiConst;  // 9, 10, 11,7 
  const ivec4 i432 =  i429 + ss.i;  // 18, 20, 22, 14

  const ivec4 i433 = ivec4(ss.i + ss.i);  // all 8s

  const ivec4 i435 = iiConst + i420;  // 9, 10, 11,7 
  const ivec4 i436 =  ss.i + i429;  // 18, 20, 22, 14

  const ivec4 i437 = i420 - i420; // 0, 0, 0, 0
  const ivec4 i438 = i420 - iiConst;  // 5, 6, 7,3 
  const ivec4 i440 =  i429 - ss.i;  // 10, 12, 14, 6

  const ivec4 i441 = ivec4(ss.i - ss.i);  // all 0s

  const ivec4 i443 = iiConst - i420;  // -5, -6, -7,-3 
  const ivec4 i444 =  ss.i - i429;  // -10, -12, -14, -6

  const ivec4 i445 = i420 * i420; // 49, 64, 81, 25
  const ivec4 i446 = i420 * iiConst;  // 14, 16, 18,10 
  const ivec4 i448 =  i429 * ss.i;  // 56, 64, 72, 40

  const ivec4 i449 = ivec4(ss.i * ss.i);  // all 16

  const ivec4 i451 = iiConst * i420;  // 14, 16, 18,10 
  const ivec4 i452 =  ss.i * i429;  // 56, 64, 72, 40

  const ivec4 i453 = i420 / i420; // 1, 1, 1, 1
  const ivec4 i454 = i420 / iiConst;  // 3, 4, 4,2 
  const ivec4 i456 =  i429 / ss.i;  // 3, 4, 4, 2

  const ivec4 i457 = ivec4(ss.i / ss.i);  // all 1s

  const ivec4 i459 = iiConst / i420;  // 0 , 0, 0,0 
  const ivec4 i460 =  ss.i / i429;  // 0, 0, 0,0 

  const bvec4 b424 = bvec4(s22.bv4);

  const bool b1 = s22.bv4 == b424;  // true
  const bool b2 = i420 == i420;  // true
  const bool b3 = i420 == i445;  // false
  const bool b4 = v420 == v420;  // true
  const bool b5 = m430 == m434; // true

  const vec4 v465 = -v420; // -7, -8, -9, -5
  const mat4 m448 = -m447 ; // diagonal -.1428, others -0.5
  const ivec4 i465 = -i456 ;  // -3, -4, -4,-2 

  const bool b7 = s22 == s22;

  const vec4 v466 = v432 + vec4(3,4,5,6);  // 21, 24, 27, 20
  const vec4 v467 = v432 + vec4(vec2(3,4),5,6);  // 21, 24, 27, 20
  const vec4 v468 = v432 + vec4(3, vec2(4, 5),vec2(6,7));  // 21, 24, 27, 20
  const vec4 v469 = vec4(v468) + vec4(3) + v468 + vec4(s77.m3[2][0]); // 45, 51, 57, 43

  const bool b8 = ss == ss;  // true

  struct st6 {
       vec3 v;
  };

  struct st5 {
      int i;
      float f;
      st6  st66;
  } st55;

  const st5 st551 = st5(2, 4.0, st6(vec3(7)));
  const st5 st552 = st5(2, 4.0, st6(vec3(7)));

  const bool b10 = st551 == st552;  // true

  const bool b11 = st551.st66 == st552.st66;  // true

  const st5 st553 = st5(2, 4.0, st6(vec3(8)));

  const bool b12 = st551.st66 == st553.st66;  // false
  const bool b13 = st551 == st553;  // false

  const bool b14 = st551 != st552;  // false
  const bool b15 = st551.st66 != st552.st66;  // false
  const bool b16 = st551.st66 != st553.st66;  // true
  const bool b17 = st551 != st553;  // true

  const bool b18 = s22.bv4 != b424;  // false
  const bool b19 = i420 != i420;  // false
  const bool b20 = i420 != i445;  // true
  const bool b21 = v420 != v420;  // false
  const bool b22 = m430 != m434; // false

  const int int10 = i420.xy.y;  // 8

  //float f = v470.x;



  const int int13 = -ss.i;
  
  const vec4 v474 = -vec4(0.5);

  int int14 = ii++;
  int array[3];
  array[2];

  const vec4 v478 = v466 * 2.0; // 42, 48, 54, 40

  const vec4 v479 = iiConst > 1 ? v466 : v478; // 21, 24, 27, 20

  const struct st7 { 
       int i; 
       bool b;
  } st77 = st7(ss.i, true);

  const vec4 v481 = vec4(st77.i);

  const struct st8 {
      int i;
  } ;


  const struct st9 {
      s2 ss;
  } st99 = st9(s22);

  const vec3 v312 = st99.ss.v3;   // 9, 10, 11
  const vec4 v482 = mat4(1)[0];  // 1, 0, 0 , 0
  
  const mat4 m450 = mat4(ss.i);  // mat4(4)
  const mat4 m451 = mat4(b20);   // mat4(1)
  const mat4 m452 = mat4(st77.b); // mat4(1)

  const vec4 v483 = vec4(vec4(3).x);  // 3,3,3,3
  const mat4 m453 = mat4(vec4(5).x);  // mat5(5)

  const vec4 v484 = vec4(mat4(6)[1]);  // 0,6,0,0
  const mat4 m454 = mat4(mat4(6)[1][1]);  // mat4(6)

  const vec4 v485 = vec4(st7(8, true).b);  // 1,1,1,1

  const vec4 v487 = vec4(vec4(12, 13, 14, 15).ab, 12, 14);

  int i20 = ss.i;
  const vec4 v489 = -vec4(7,8,9,5); // -7, -8, -9, -5

  gl_Position = vec4(1);
}
