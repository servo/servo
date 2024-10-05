
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
//
// mat3arrayindirect1_frag.frag: Fragment shader solid color  testing indirect referencing into uniforms
// The vec3 values are determined at runtime.
//
//

uniform mat3 testmat3[2];
varying vec4  color;

void main(void)
{
	vec3 result = vec3(0.0, 0.0, 0.0);

  /*
	// No indirect indexing in fragment shaders
	for(int j = 0; j < 3; j++)
	{
		result += testmat3[1][j];
	}
*/
	result += testmat3[1][0];
	result += testmat3[1][1];
	result += testmat3[1][2];
	gl_FragColor = vec4(result/2.0, 0.5);
}
