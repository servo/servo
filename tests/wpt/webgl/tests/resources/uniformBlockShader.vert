#version 300 es

/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/

in vec4 a_vertex;
in vec3 a_normal;

uniform Transform {
    mat4 u_modelViewMatrix;
    mat4 u_projectionMatrix;
    mat3 u_normalMatrix;
};

out vec3 normal;
out vec4 ecPosition;

void main()
{
    normal = normalize(u_normalMatrix * a_normal);
    ecPosition = u_modelViewMatrix * a_vertex;
    gl_Position =  u_projectionMatrix * ecPosition;
}
