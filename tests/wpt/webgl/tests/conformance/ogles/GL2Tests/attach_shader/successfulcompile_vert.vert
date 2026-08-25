
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec3 gtf_Normal;
attribute vec4 gtf_Vertex;
uniform mat3 gtf_NormalMatrix;
uniform mat4 gtf_ModelViewMatrix;
uniform mat4 gtf_ModelViewProjectionMatrix;

varying float lightIntensity;
varying vec3 Position;
uniform vec3 LightPosition;
uniform float Scale;

void main(void) {
	vec4 pos = gtf_ModelViewMatrix * gtf_Vertex;
	Position = vec3(gtf_Vertex) * Scale;
	vec3 tnorm = normalize(gtf_NormalMatrix * gtf_Normal);
	lightIntensity = dot(normalize(LightPosition - vec3(pos)), tnorm) * 1.5;
	gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
}
