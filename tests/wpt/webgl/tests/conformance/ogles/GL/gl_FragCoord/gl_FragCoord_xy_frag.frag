
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform float viewportwidth;
uniform float viewportheight;

void main(void)
{
	// The image width is 500 so scale the position to 0...1 for color
	gl_FragColor = vec4(gl_FragCoord.x /viewportwidth , gl_FragCoord.y/viewportheight, 0.0, 1.0);
}
