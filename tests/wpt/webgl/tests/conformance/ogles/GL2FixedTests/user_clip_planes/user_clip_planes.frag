
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif

varying vec4 color;
varying float dotClip[2];

void main (void) 
{
	if (dotClip[0] >= 0.0 || dotClip[1] >= 0.0)
		discard;
		
    gl_FragColor = color;
}
