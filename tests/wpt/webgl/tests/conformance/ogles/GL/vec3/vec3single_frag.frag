
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
//
// vec3Matrix_frag.frag: Fragment shader solid color
//
//
//

uniform vec3 lightPosition;
varying vec4  color;

void main(void)
{
     gl_FragColor = vec4(lightPosition, 0.0);
}
