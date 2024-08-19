
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


attribute vec4 gtf_Normal;
void main()
{
    gtf_Normal = vec3(1.0,2.0,3.0);  // cannot be modified an attribute
}
