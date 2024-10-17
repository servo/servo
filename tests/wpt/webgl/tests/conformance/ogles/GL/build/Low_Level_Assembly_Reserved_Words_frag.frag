
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform float MIN;
uniform float R0;
uniform float FOGC;
uniform float CUBE;
uniform float f;
uniform float o;
uniform float p;
uniform float w;
uniform float x;
uniform float y;
uniform float z;

void main()
{
	gl_FragColor = vec4(f, o, p, w);
}
