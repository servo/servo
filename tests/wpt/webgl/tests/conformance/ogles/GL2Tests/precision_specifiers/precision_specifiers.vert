
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute highp vec4 gtf_Color;
attribute highp vec4 gtf_Vertex;
uniform highp mat4 gtf_ModelViewProjectionMatrix;
varying highp vec4 color;

void main (void)
{
	mediump int x = 5;
	lowp int y = 3;
	mediump float x2 = 5.0;
	lowp float y2 = 1.0;
	
	color = vec4(x + y, x2 * y2, x, 1.0);
	
	color = gtf_Color;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
