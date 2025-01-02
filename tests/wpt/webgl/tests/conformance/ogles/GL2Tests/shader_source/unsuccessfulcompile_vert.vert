
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec3 gtf_Normal;
attribute vec4 gtf_Vertex;
attribute vec4 gtf_Vertex_Color;
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
    vec4 pos        = gtf_ModelViewMatrix * gtf_Vertex_Color;
    Position        = vec3(gtf_Vertex);
    vec3 tnorm      = normalize(gtf_NormalMatrix * gtf_Normal);
    vec3 lightVec   = normalize(LightPosition - vec3(pos);
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
