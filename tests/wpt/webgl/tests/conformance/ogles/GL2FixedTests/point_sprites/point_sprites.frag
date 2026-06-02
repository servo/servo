
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


uniform sampler2D gtf_Texture0;

void main (void)
{
	gl_FragColor = texture2D(gtf_Texture0, gl_PointCoord.st);
}
