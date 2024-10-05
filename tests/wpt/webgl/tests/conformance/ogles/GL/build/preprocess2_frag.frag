
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
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



