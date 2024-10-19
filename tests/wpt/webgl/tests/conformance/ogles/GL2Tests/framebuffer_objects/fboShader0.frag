
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif

varying vec4 color;
varying vec4 texCoord[1];

uniform sampler2D gtf_Texture0;
uniform int gtf_UseTexture;

void main (void)
{
    if ( gtf_UseTexture == 1 )
    {
        gl_FragColor = texture2D(gtf_Texture0, texCoord[0].xy);
    }
    else
    {
        gl_FragColor = color;
    }
}
