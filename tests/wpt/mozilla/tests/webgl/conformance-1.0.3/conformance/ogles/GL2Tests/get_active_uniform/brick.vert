
/*
** Copyright (c) 2012 The Khronos Group Inc.
**
** Permission is hereby granted, free of charge, to any person obtaining a
** copy of this software and/or associated documentation files (the
** "Materials"), to deal in the Materials without restriction, including
** without limitation the rights to use, copy, modify, merge, publish,
** distribute, sublicense, and/or sell copies of the Materials, and to
** permit persons to whom the Materials are furnished to do so, subject to
** the following conditions:
**
** The above copyright notice and this permission notice shall be included
** in all copies or substantial portions of the Materials.
**
** THE MATERIALS ARE PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
** EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
** MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
** IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
** CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
** TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
** MATERIALS OR THE USE OR OTHER DEALINGS IN THE MATERIALS.
*/


attribute vec3 gtf_Normal;
attribute vec4 gtf_Vertex;
varying float lightIntensity;
varying vec3  Position;

                                             // Used in the vertex shader.
uniform mat3 gtf_NormalMatrix;               //<  1
uniform mat4 gtf_ModelViewMatrix;            //<  2
uniform mat4 gtf_ModelViewProjectionMatrix;  //<  3
uniform float myAttrib1f;                    //<  4
uniform vec2 myAttrib2f;                     //<  5
uniform vec3  LightPosition;                 //<  6
uniform vec4 myAttrib4f;                     //<  7
uniform int myAttrib1i;                      //<  8
uniform ivec2 myAttrib2i;                    //<  9
uniform ivec3 myAttrib3i;                    //< 10
uniform ivec4 myAttrib4i;                    //< 11
uniform bool myAttrib1b;                     //< 12
uniform bvec2 myAttrib2b;                    //< 13
uniform bvec3 myAttrib3b;                    //< 14
uniform bvec4 myAttrib4b;                    //< 15
uniform mat2 myAttrib2m;                     //< 16
uniform mat3 myAttrib3m;                     //< 17
uniform mat4 myAttrib4m;                     //< 18
uniform float myUniformfv[5];                //< 19
                                             // Used in the fragment shader.
uniform vec3	brickColor;                  //< 20
uniform vec3	mortarColor;                 //< 21
uniform float	brickMortarWidth;            //< 22
uniform float	brickMortarHeight;           //< 23
uniform float	mwf;                         //< 24
uniform float	mhf;                         //< 25


const float specularContribution = 0.7;
const float diffuseContribution  = (1.0 - specularContribution);

void main(void) {
    vec4 pos        = gtf_ModelViewMatrix * gtf_Vertex;
    Position        = vec3(gtf_Vertex);
    vec3 tnorm      = normalize(gtf_NormalMatrix * gtf_Normal);
    vec3 lightVec   = normalize(LightPosition - vec3(pos));
    vec3 reflectVec = reflect(lightVec, tnorm);
    vec3 viewVec    = normalize(vec3(pos));

	float f = myAttrib1f + myAttrib2f[0] + myAttrib4f[0] 
			  + float(myAttrib1i) + float(myAttrib2i[0]) + float(myAttrib3i[0]) + float(myAttrib4i[0])
			  + float(myAttrib1b) + float(myAttrib2b[0]) + float(myAttrib3b[0]) + float(myAttrib4b[0])		  
			  + myAttrib2m[0][0] + myAttrib3m[0][0] + myAttrib4m[0][0]
			  + myUniformfv[0] + myUniformfv[1] + myUniformfv[2] + myUniformfv[3] + myUniformfv[4];

	//float spec = clamp(dot(reflectVec, viewVec), 0.0, 1.0);
	float spec = clamp(dot(reflectVec, viewVec), f, 1.0);
    spec = spec * spec;
    spec = spec * spec;
    spec = spec * spec;
    spec = spec * spec;

    lightIntensity = diffuseContribution * dot(lightVec, tnorm) +
                     specularContribution * spec;
    
    gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
