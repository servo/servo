
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void main()
{
    const vec4 v = vec2(2.0, 3.0);
    gl_FragColor = v;
}
