
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
void main(void)
{
	gl_FragColor = vec4(vec3(gl_FragCoord.w), 1.0);
}
