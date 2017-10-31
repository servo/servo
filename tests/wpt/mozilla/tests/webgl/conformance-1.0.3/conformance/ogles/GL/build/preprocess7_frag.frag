
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


#ifdef GL_ES
precision mediump float;
#endif
// testing for char constants in #if and #elif
// Also checking whether reserved words can be redefined.

#define t1 c
#define t2 d
#define asm a

 #if(t1==c)
  #define t3 3
 #elif(t1==d)
  #define t4 4
 #elif(t2==c)
  #define t5 5
 #endif

 #ifndef t1
   #define t7 7
 #elif (t2==d)
  #define t6 6
 #endif

 #if (t2=='d')
  #define half 5
 #else
  #define half 8
 #endif

 #ifdef t22
  #define x 5
 #endif

 void main(void)
  {
   int sum =0,a=9;

   sum = half + sum;
   sum = asm + a;

  }

