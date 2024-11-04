
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform sampler2DRect samp;

void main()
{
	gl_FragColor = texture2DRect(samp, vec2(0.0, 0.0));
}
