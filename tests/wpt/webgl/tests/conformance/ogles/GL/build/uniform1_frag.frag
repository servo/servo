
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct gtf_FogParameters {
vec4 color;
float density;
float start;
float end;
float scale;
};
uniform gtf_FogParameters gtf_Fog;
void main()
{
    gtf_Fog.density = 1.0;  // cannot modify a uniform
}
