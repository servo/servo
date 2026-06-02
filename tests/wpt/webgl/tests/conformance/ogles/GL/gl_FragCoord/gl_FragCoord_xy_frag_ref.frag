
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
varying vec4 color;

void main(void)
{
	// The image width is 500x500 and the rectangle is 434x434
	// The green component corresponds to x (0...1 left to right) and the
	// blue component corresponds to y (0...1 bottom to top)
	gl_FragColor = vec4((434.0 / 500.0) * (color.gb - 0.5) + 0.5, 0.0, 1.0);
}
