
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
struct gtf_PointParameters {
float size;
float sizeMin;
float sizeMax;
float fadeThresholdSize;
float distanceConstantAttenuation;
float distanceLinearAttenuation;
float distanceQuadraticAttenuation;
};
uniform gtf_FogParameters gtf_Point;
void main()
{
    gtf_PointSize = 4.0;  // can be used in vertex shader only
}
