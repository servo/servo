
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

void main (void)
{
	bool toDiscard = false;
	if(color.r > 0.75) toDiscard = true;
	else if(color.g > 0.75) toDiscard = true;
	else if(color.b > 0.75) toDiscard = true;

	if (toDiscard) discard;

	gl_FragColor = color;
}
