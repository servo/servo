
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
uniform sampler2D gtf_Texture0;
varying vec4 color;
varying vec4 gtf_TexCoord[1];

void main (void)
{
	gl_FragColor = texture2D(gtf_Texture0, gtf_TexCoord[0].st, 1.0) * color;
}
