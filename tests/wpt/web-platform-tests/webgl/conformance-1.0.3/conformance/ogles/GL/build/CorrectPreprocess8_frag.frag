
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
// Extensive testing on #if #else #elif #ifdef, #ifndef and #endif.


#define t1 1
 
#if(t1==1)
  #define t2 2
#endif

#if (t2!=2)
  #define t3 33
#else
  #define t3 3
#endif

#if (t3!=3)
 #define t4 4
#elif (t3==3)
 #define t4 44
#else 
  #define t4 0
#endif

#if defined(t5)
 #define t6 6
#elif (t3!=3)
 #define t5 5
#elif (t3==3)
 #define t5 5
#endif

#ifdef t5
 #define t6 6
#else
 #define t7 7
#endif

#ifndef t8 
 #define t8 8
#endif

#if defined t8 
 #define t9
 #ifdef t9
  #define  t10 10
 #endif
#elif
 #define t11 11
#endif

#ifndef t8
 #define t12 12
#else
 #define t12 12
 #ifndef t13
  #define t13 13
 #endif
 #ifdef t14
  #define t15 15
 #else
  #if defined t8
   #define t16 16
  #endif
 #endif
#endif

#ifdef t1
   #ifdef t10
      #if defined t8
         #if defined(t3)
               #ifndef t20
                  #define t25 25
               #endif
         #else
            #define t15 15
            #define t24 24
         #endif
      #endif   
   #endif
#else
   #ifdef t21
     #define t22 22
   #else
     #define t23 23
   #endif
#endif
#define t7 7
#define t11 11
#define t14 14
#define t15 15
#define t20 20
#define t22 22
#define t23 23
#define t24 42

void main(void)
{
 int sum =0;
 sum = t1+t2+t3+t4+t5; 
 sum = t6+t7+t8+t9+t10;
 sum = t11+t12+t13+t14+t15;
 sum = t16+t20+t22+t23+t25+t24;
}         

