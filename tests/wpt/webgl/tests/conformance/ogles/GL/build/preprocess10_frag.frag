
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
/* The program should terminate with an error message and not get into an
   infinite loop */
#ifdef name

void main()
{
   gl_FragColor = vec4(1);
}
