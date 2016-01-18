
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


struct gtf_MaterialParameters
{
vec4 emission;
vec4 ambient;
vec4 diffuse;
vec4 specular;
float shininess;
};
struct gtf_LightSourceParameters
{
vec4 ambient;
vec4 diffuse;
vec4 specular;
vec4 position;
vec4 halfVector;
vec3 spotDirection;
float spotExponent;
float spotCutoff;
float spotCosCutoff;
float constantAttenuation;
float linearAttenuation;
float quadraticAttenuation;
};
struct gtf_PointParameters {
float size;
float sizeMin;
float sizeMax;
float fadeThresholdSize;
float distanceConstantAttenuation;
float distanceLinearAttenuation;
float distanceQuadraticAttenuation;
};
struct gtf_DepthRangeParameters {
float near;
float far;
float diff;
};
struct gtf_LightModelParameters {
vec4 ambient;
};
struct gtf_LightModelProducts {
vec4 sceneColor;
};
struct gtf_LightProducts {
vec4 ambient;
vec4 diffuse;
vec4 specular;
};
struct gtf_FogParameters {
vec4 color;
float density;
float start;
float end;
float scale;
};
uniform int gtf_MaxFragmentUniformComponents;
uniform int gtf_MaxVertexUniformComponents;
uniform int gtf_MaxVertexTextureImageUnits;
uniform int gtf_MaxLights;
uniform int gtf_MaxClipPlanes;
uniform int gtf_MaxCombinedTextureImageUnits;
uniform int gtf_MaxTextureCoords;
uniform int gtf_MaxVertexAttribs;
uniform int gtf_MaxVaryingFloats;
uniform int gtf_MaxTextureUnits;
uniform int gtf_MaxDrawBuffers;
uniform int gtf_MaxTextureImageUnits;
uniform gtf_LightProducts gtf_FrontLightProduct[8];
uniform gtf_LightModelProducts gtf_FrontLightModelProduct;
uniform gtf_DepthRangeParameters gtf_DepthRange;
uniform gtf_FogParameters gtf_Fog;
uniform gtf_PointParameters gtf_Point;
uniform gtf_LightModelParameters gtf_LightModel;
varying vec4 gtf_FogFragCoord;
varying vec4 gtf_BackColor;
varying vec4 gtf_BackSecondaryColor;
varying vec4 gtf_FrontSecondaryColor;
varying vec4 gtf_TexCoord[2];
varying vec4 gtf_FrontColor;
uniform gtf_MaterialParameters gtf_FrontMaterial;
uniform gtf_LightSourceParameters gtf_LightSource[8];
attribute vec4 gtf_MultiTexCoord1;
attribute vec4 gtf_MultiTexCoord2;
attribute vec4 gtf_SecondaryColor;
attribute vec4 gtf_Color;
attribute vec4 gtf_MultiTexCoord3;
attribute vec4 gtf_MultiTexCoord0;
attribute vec4 gtf_Normal;
attribute vec4 gtf_Vertex;
uniform mat4 gtf_NormalMatrix;
uniform mat4 gtf_ProjectionMatrix;
uniform mat4 gtf_TextureMatrix[8];
uniform mat4 gtf_ModelViewMatrix;
uniform mat4 gtf_ModelViewProjectionMatrix;
void test_function(const in int in_int, inout int out_int);
int test_function1(in int in_int1, inout int in_out_int);

uniform float array_float[2]; 

struct nested
{
   int a;
   float f; 
};

struct light1 
{
   float intensity;
   vec3 position;
   int test_int[2];
   nested light2;
} lightVar;
light1 ll2;

void Assign (out light1 out1, in light1 in1)
{
    out1.intensity = in1.intensity;
     out1.position = in1.position;
  out1.test_int[0] = in1.test_int[0];
  out1.test_int[1] = in1.test_int[1];
       out1.light2 = in1.light2;
}

struct light3 {
    float i;
};

struct light4 {
    float i;
};

struct light5 {
    float i ;
    float a[2];
} light5_inst;

uniform light3 uniformLight3;

struct light6 {  
    float i;
};
uniform light6 uniformLight6; 

struct slight10{
     float f;
     };
struct slight9{
     slight10 light10;
     };
struct slight8{
     slight9 light9;
     };
struct light7 {
  slight8 light8;
} ;


light3 struct_var = light3(5.0); 

// Attribtue variables can only be Global
attribute float flt_attrib;
attribute vec2 vec2_attrib;
attribute vec3 vec3_attrib;
attribute vec4 vec4_attrib; 
attribute mat2 mat2_attrib; 
attribute mat3 mat3_attrib; 
attribute mat4 mat4_attrib; 

uniform float flt_uniform; 
uniform vec3 uniform_vec3; 
uniform mat3 uniform_mat3; 

uniform sampler2D samp[3];  
uniform sampler2D samp1;  

const struct light12 { 
    int a;
} uniform_struct = light12(2);

varying vec3 varying_vec3; 
varying vec2 varying_vec2;  
varying vec4 varying_vec4;  
varying mat4 varying_mat4;  
varying mat2 varying_mat2;  
varying mat3 varying_mat3;  
varying float varying_flt;  

float frequencies[2]; 

void test_function2(int func_int)
{
}

void test_function3(light3);
void test_function4(light5 ll20);
void test_function5(light1);
light6 test_function6(int a);

const float FloatConst1 = 3.0 * 8.0, floatConst2 = 4.0;
const bool BoolConst1 = true && true || false; 
const bool BoolConst2 = false || !false && false; 

void main(void)
{

    int test_int1 = 2; 
    const int const_test_int1 = 2; 

    struct structMain {
        float i;
    } testStruct;

    struct {    
        structMain a;
    } aStruct;

    testStruct.i = 5.0 ; 
    struct_var.i = 5.0;  

    structMain newStruct, newStruct1;
    testStruct = newStruct; 
    newStruct = newStruct1;  

    lightVar.light2.f = 1.1; 

    light1 ll1; 
    ll1.light2.a = 1;  

     const struct const_struct {
        float i;
    } const_struct_inst = const_struct(1.0); 

    //ll1 = ll2; 
    Assign (ll1, ll2); 
    ll1.light2 = ll2.light2; 
    ll1.light2 = ll1.light2; 
    ll1.light2.f = ll2.light2.f;
    ll1.light2.f = ll1.light2.f;

    //    lightVar = ll2;
    //    ll2 = lightVar;
    Assign (lightVar, ll2);
    Assign (ll2, lightVar);

    light5 ll10;

    light7 ll7[4];
    structMain newStruct2[2];
    newStruct2[0].i = 1.1; 
    
    ll7[0].light8.light9.light10.f = 1.1;


    bool test_bool4 = false ; 

    bool test_bool5 = 1.2 > 3.0 ; 

    int test_int2 =  047; 
    int test_int4 =  0xa8;  // testing for hexadecimal numbers

    float test_float1 = 1.5; 
    float test_float2 = .01;  
    float test_float3 = 10.; 
    float test_float4 = 10.01; 
    float test_float5 = 23e+2; 
    float test_float6 = 23E-3; 
    float test_float8 = 23E2; 
    bool test_bool6 = BoolConst1 && ! (test_int1 != 0) && ! BoolConst1  && ! (FloatConst1 != 0.0) && (FloatConst1 != 0.0) && (test_float1 != 0.0); 

    vec4 color = vec4(0.0, 1.0, 0.0, 1.0); 
    vec4 color2 = vec4(0.0); 

    vec3 color4 = vec3(test_float8); 

    ivec4 test_int_vect1 = ivec4(1.0,1.0,1.0,1.0);  
    ivec3 test_int_vec3 = ivec3(1, 1, 1) ; 

    bvec4 test_bool_vect1 = bvec4(1., 1., 1. , 1. ); 

    vec2 test_vec2 = vec2(1., 1.); 
    vec2 test_vec3 = vec2(1., 1);  
    vec4 test_vec4 = vec4(test_int_vect1); 

    vec2 test_vec5 = vec2(color4);
    vec3 test_vec7 = vec3(color);   
    vec3 test_vec8 = vec3(test_vec2, test_float4);
    vec3 test_vec9 = vec3(test_float4, test_vec2);

    vec4 test_vec10 = vec4(test_vec9, 0.01); 
    vec4 test_vec11 = vec4(0.01, test_vec9); 

    vec4 test_vec12 = vec4(test_vec2, test_vec2); 

    mat2 test_mat2 = mat2(test_float3); 
    mat3 test_mat3 = mat3(test_float3); 
    mat4 test_mat4 = mat4(test_float3); 

    mat2 test_mat7 = mat2(test_vec2, test_vec2); 
    mat2 test_mat8 = mat2(01.01, 2.01, 3.01, 4.01); 

    mat3 test_mat9 = mat3(test_vec7, test_vec7, test_vec7); 
    mat4 test_mat10 = mat4(test_vec10, test_vec10, test_vec10, test_vec10); 
    test_mat10[1] = test_vec10; 
    

    mat2 test_mat12 = mat2(test_vec2, 0.01, 0.01); 
    mat2 test_mat13 = mat2(0.01, 5., test_vec2); 
    mat2 test_mat15 = mat2(0.1, 5., test_vec2 ); 

    //mat2 test_mat16 = mat2(test_mat9); 
    //mat2 test_mat17 = mat2(test_mat10); 

    float freq1[2]; 
    float freq2[25]; 

    for (int i=0; i<100; i++)
    {
      if (test_float1 < 1.0)
      {
        
      }
      else
      {
        break;
      }
    }
    
    freq2[1] = 1.9 ; 
    const int array_index = 2;
    freq2[const_test_int1] = 1.9 ;
    freq2[array_index] = 1.8;
    
    const int const_int = 5; 
   
    test_float1 = varying_flt; 

    int out_int;
    int intArray[6];
    test_function(test_int1, test_int1); 
    test_function(test_int1, intArray[2]); 

    vec3 vv = vec3(test_function1(test_int1, out_int));  
    bool bool_var = true;
    int test_int6 = int(bool_var); 
    test_float1 = float(bool_var); 
    test_float1 = float(test_int6); 
    test_int6 = int(test_float1); 
    bool_var = bool(test_int6); 
    bool_var = bool(test_float1); 
    test_float1 = float(test_vec9); 
    
    test_vec2.x = 1.2; 
    test_vec2.y = 1.4; 
    test_vec2.xy; 


    color.zy = test_vec2; 

   test_vec2[1] = 1.1;  
    
     test_mat2[0][0] = 1.1; 

    test_float1 += 1.0; 
    test_float1 -= 1.0;
    test_float1 *= 1.0;
    test_float1 /= 1.0;

    test_mat12 *= test_mat13 ; 
    test_mat12  *= test_float1;
    test_vec2 *= test_float1; 
    test_vec2 *= test_mat12; 
    test_float1++; 
    test_float1--; 
    --test_float1; 
    ++test_float1; 
    test_float1; 
    test_int1++; 
    test_int1--; 

    test_vec2 = test_vec2 + test_float1;   
    test_vec2 = test_float1 + test_vec2;   

    test_mat12 = test_mat12 * test_mat13; 
    test_vec2 = test_vec2 * test_vec5; 
 
    test_vec2++; 
    test_mat2++;

    bool test_bool2 = test_float2 > test_float3;  

    bool test_bool3 = test_int1 > test_int6 ; 

    test_bool3 = test_vec2 == test_vec5; 

    test_bool2 = test_bool3 && test_bool4; 
    test_bool2 = test_bool3 || test_bool4; 
    test_bool2 = test_bool3 ^^ test_bool4; 

    test_bool2 = !test_bool3;  

    test_bool3 = !(test_int1 > test_int6) ; 

    test_float1 = test_int1 > test_int6 ? test_float2 : test_float3;  
    test_vec2 = test_int1 > test_int6 ? test_vec2 : test_vec5;  
    if(test_bool2)  
        test_float1++;
    else
	test_float1--;

    if(test_float1 > test_float2)  
        test_float1++;

    if( test_bool2 )  
    {
        int if_int; 
        test_float1++;
    }

    if(test_bool2) 
       if(test_bool3)
           if(test_bool3)
	      test_float1++;

   for(int for_int=0; for_int < 5; for_int++) 
   {
       // do nothing as such
   }


   for(int x1=0; x1 < 10; x1++) 
   {
     if (!test_bool2)
       break;
       
     int for_int;
   }

   for(int x2=-10; x2 < 100; x2++) 
   {
     test_bool2 = (test_float1 > test_float2);
     if (!test_bool2)
       break;
   }

   for(int for_int1 = 0; for_int1 < 100; for_int1++) 
   {
     if (!test_bool2)
       break;
       
     int for_int;
   }

   for(int for_int1 = 0; for_int1 < 100; for_int1++) 
   {
     if (!test_bool2)
       continue;
       
     int for_int;
   }


   for(int i=0; i<100; i++) 
   {
     if (!(test_float1 > test_float2))
     {
       break;
     }
     
     break;
     continue;  
   }

   for(int i=0; i<100; i++)  
   {
     if (!test_bool2)
       break;
       
     break;  
   }

   for (int i=0; i<100; i++)
   {
     int dowhile_int;
     dowhile_int = 3;

     if (!test_bool2)
       break;
   }

    gl_Position = vec4(2.0, 3.0, 1.0, 1.1);
    gl_Position = gtf_Vertex;


    // VERTEX SHADER BUILT-IN ATTRIBUTES

    vec4 builtInV4 = gtf_Color + gtf_SecondaryColor + gtf_Vertex + gtf_MultiTexCoord0 + gtf_MultiTexCoord1 + gtf_MultiTexCoord2 +  gtf_MultiTexCoord3;
    

    int builtInI = gtf_MaxLights + gtf_MaxClipPlanes + gtf_MaxTextureUnits + gtf_MaxTextureCoords + gtf_MaxVertexAttribs + gtf_MaxVertexUniformComponents + gtf_MaxVaryingFloats + gtf_MaxVertexTextureImageUnits + gtf_MaxCombinedTextureImageUnits + gtf_MaxTextureImageUnits + gtf_MaxFragmentUniformComponents + gtf_MaxDrawBuffers ;
    

    mat4 builtInM4 = gtf_ModelViewMatrix + gtf_ModelViewProjectionMatrix + gtf_ProjectionMatrix;

    gtf_NormalMatrix;

    gtf_TextureMatrix[gtf_MaxTextureCoords-1];
    gtf_TextureMatrix;

    gtf_DepthRange.near ;

    test_float1 = gtf_DepthRange.near; 
    test_float1 = gtf_DepthRange.far; 
    test_float1 = gtf_DepthRange.diff;

    gtf_Point.size; 
    gtf_Point.sizeMin;
    gtf_Point.sizeMax; 
    gtf_Point.fadeThresholdSize ;
    gtf_Point.distanceConstantAttenuation;
    gtf_Point.distanceLinearAttenuation ;
    gtf_Point.distanceQuadraticAttenuation;

    gtf_MaterialParameters test; 
    gtf_FrontMaterial.emission;

    color = gtf_FrontMaterial.emission; 
    color = gtf_FrontMaterial.ambient; 
    color = gtf_FrontMaterial.diffuse;
    color = gtf_FrontMaterial.specular;
    test_float1 = gtf_FrontMaterial.shininess; 

    gtf_LightSourceParameters lightSource;

    float builtInFloat1 = gtf_LightSource[0].spotExponent;
    color = gtf_LightSource[0].ambient; 
    color = lightSource.ambient; 
    color = lightSource.diffuse; 
    color = lightSource.specular; 
    color = lightSource.position; 
    color = lightSource.halfVector; 
    color4 = lightSource.spotDirection; 
    test_float1 = lightSource.spotExponent; 
    test_float1 = lightSource.spotCutoff; 
    test_float1 = lightSource.spotCosCutoff; 
    test_float1 = lightSource.constantAttenuation; 
    test_float1 = lightSource.linearAttenuation; 
    test_float1 = lightSource.quadraticAttenuation; 

    color = gtf_LightModel.ambient;

    gtf_LightModelParameters lightModel; 
    color = gtf_LightModel.ambient; 
    color = lightModel.ambient; 

    color = gtf_FrontLightModelProduct.sceneColor ;

    gtf_LightModelProducts lightModelProd; 

    color = lightModelProd.sceneColor; 
    color = gtf_FrontLightModelProduct.sceneColor; 

    color = gtf_FrontLightProduct[0].ambient; 
    color = gtf_FrontLightProduct[0].ambient; 
    gtf_LightProducts lightProd;

    color =  lightProd.ambient; 
    color =  lightProd.diffuse;
    color =  lightProd.specular;


    test_float1 = gtf_Fog.density ;
    test_float1 = gtf_Fog.start ;
    test_float1 = gtf_Fog.end  ;
    test_float1 = gtf_Fog.scale ;
    color = gtf_Fog.color ;

    gtf_FrontColor =  vec4(1.0, 1.0, 1.0, 1.0); 
    gtf_BackColor =  vec4(1.0, 1.0, 1.0, 1.0);  
    gtf_FrontSecondaryColor =  vec4(1.0, 1.0, 1.0, 1.0); 
    gtf_BackSecondaryColor =  vec4(1.0, 1.0, 1.0, 1.0); 


    // VARYING VARIABLES AVAILABLE IN FRAGMENT AND VERTEX SHADERS BOTH
    gtf_TexCoord[0] =  vec4(1.0, 1.0, 1.0, 1.0);  
    gtf_FogFragCoord =  vec4(1.0, 1.0, 1.0, 1.0);  

}

void test_function(const in int in_int, inout int out_int)
{
    out_int = 5; 
    int i = 5;
    return ;
}

int test_function1(in int in_int1, inout int in_out_int)
{
   float ff;
   in_int1 = 5;  
   return in_int1;
}

void test_function3(light3 ll)
{
    ll.i = 5.0;  
    varying_flt = 1.2;
}

void test_function4(light5 ll20)
{
    ll20.i = 10.0; 
}

void test_function5(light1 struct_light1)
{
    struct_light1.light2.a = 1; 
    light5 ll5;
    struct_light1.light2.f = ll5.i;
    struct_light1.light2.f++;
    struct_light1.light2.a++;
}

light6 test_function6(int a)  
{
    int x;
    light6 funcStruct;
    light7 funcStruct1;
    -x;
    x = x - x ; 
    mat2 m;
    m++;
    -m; 
    (m)++; 
    return funcStruct; 
}

float test_function7(light1 ll1, int light1 )  
{
    float f;
    
    struct ss1 {
        int a;
    };

    return float(1);
}
