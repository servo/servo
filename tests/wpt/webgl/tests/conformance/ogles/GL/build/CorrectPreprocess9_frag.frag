
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


#ifdef GL_ES
precision mediump float;
#endif
#define t1 2.3333333333333333
#define t2 (0.978293600-1.0)
#define t3 .9090909090
#define t4 26578235.000000083487
#define t5 78e-03
#define t6 78.100005E+05
#define t7 6278.78e-5

void main(void){
    float tes=2e-3;
    float test=3.2e-5;
    float test1=0.99995500;
    float test2=6789.983;

    test = t1+t2;
    test = t3-t4;
    tes  = t5 * t6;
    test2 = t7;

    gl_FragColor = vec4(test, tes, test1, test2);
}
