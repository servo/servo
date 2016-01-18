
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
// #line directive-- test cases.
// chks for Invalid directives, all possible #line errors
// Also checks the correct verions of #line dorective.

#define t1 1
#define t2 2

#
#
#
#
#line 8
#line "" 
#line 3 3

#linekfj
#line c c 
#line t1 t2
#line 77 89
#line 65.4 
#line message to the user
#line
#line345

void main(void)
{
 int sum =1;
 sum = __LINE__;
 sum = __FILE__;
 #line 4 5
 sum = __LINE__;
 sum = __FILE__;
 #line 9
 sum = __LINE__ + __FILE__ ;
 sum = __FILE__;
 #
 #
 sum = __VERSION__;
 sum = sum + __LINE__ ;
 #line 4 5
 #line 5 8
 sum = __LINE__;
 sum = __FILE__;
 sum = __VERSION__;

}

 

