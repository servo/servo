
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


/* This is  a comment*/ int i; // This is a global decl
#version 100  // error #version should be the first statement in the program
#ifdef GL_ES
precision mediump float;
#endif


void main()
{
   gl_FragColor = vec4(1);
}
