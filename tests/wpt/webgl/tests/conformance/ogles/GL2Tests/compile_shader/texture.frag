
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
//
// wobble.frag: Fragment shader for wobbling a texture
//
// author: Antonio Tejada
//
//

varying vec3  Position;
varying float lightIntensity;

/* Constants */

uniform sampler2D sampler2d; // value of sampler2d = 0
varying vec4 gtf_TexCoord[1];

void main (void)
{
    vec3 lightColor = vec3(texture2D(sampler2d,  vec2(gtf_TexCoord[0]))) * lightIntensity;

    vec3 ct = clamp(lightColor, 0.0, 1.0);

    gl_FragColor = vec4 (ct, 1.0);
}

