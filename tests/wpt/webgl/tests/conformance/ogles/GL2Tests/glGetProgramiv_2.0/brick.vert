
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
uniform mat3 gtf_NormalMatrix;
uniform mat4 gtf_ModelViewMatrix;
uniform mat4 gtf_ModelViewProjectionMatrix;
varying float lightIntensity;
varying vec3  Position;
uniform vec3  LightPosition;
uniform vec3  NotActiveOne;
attribute float myAttribute1;
attribute float myAttribute2;

const float specularContribution = 0.7;
const float diffuseContribution  = (1.0 - specularContribution);

void main(void) {
    vec4 pos        = gtf_ModelViewMatrix * gtf_Vertex;
    Position        = vec3(gtf_Vertex);
    vec3 tnorm      = normalize(gtf_NormalMatrix * gtf_Normal);
    vec3 lightVec   = normalize(LightPosition - vec3(pos));
    vec3 reflectVec = reflect(lightVec, tnorm);
    vec3 viewVec    = normalize(vec3(pos));

	//float spec = clamp(dot(reflectVec, viewVec), 0.0, 1.0);
	float spec = clamp(dot(reflectVec, viewVec), myAttribute1, myAttribute2);
    spec = spec * spec;
    spec = spec * spec;
    spec = spec * spec;
    spec = spec * spec;

    lightIntensity = diffuseContribution * dot(lightVec, tnorm) +
                     specularContribution * spec;
    
    gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
