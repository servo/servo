
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
// tests for macro redifinition (t2) and the #if and #else nestings.
// takes care of elif also. 

#define t1 (1+2) 
#define t2 2
#define t2 3 

// testing the if depth
#if (t1==3)
  #define t3 3
  #if defined t2
    #define t4 4
      #if defined(t3)
          #define t5 5
             #ifdef t5
               #define t6 6
                 #ifndef t7
                   #define t7 7
                 #else
                   #define t8 8
                 #endif
             #endif
      #else
         #ifndef t8
             #define t8 8
         #elif (t8==8)
            #define t9 9
         #else
            #if defined t7
              #define t9 9
            #endif
         #endif
      #endif
  #else
    #define t10 10
  #endif
#endif


#define t8 8 
#define t9 9 
#define t10 10

void main(void) 
{
 int sum=1 ;
 sum = t1+t2;
 sum = t3+t4;
 sum = t5+t6;
 sum = t7+t8;
 sum = t9+t10;
}    
              
