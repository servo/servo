
/*
Copyright (c) 2019 The Khronos Group Inc.
Use of this source code is governed by an MIT-style license that can be
found in the LICENSE.txt file.
*/


uniform mat4 gtf_ModelViewMatrix;
uniform mat4 gtf_ModelViewProjectionMatrix;
uniform mat3 gtf_NormalMatrix;

attribute vec4 gtf_Vertex;
attribute vec4 gtf_Color;
attribute vec3 gtf_Normal;

varying vec4 color;

vec4 Ambient;
vec4 Diffuse;
vec4 Specular;

const vec3 lightPosition = vec3(0.0, 0.0, 10.0);
const float lightAttenuationConstant = 1.0;
const float lightAttenuationLinear = 0.0;
const float lightAttenuationQuadratic = 0.0;

const vec4 lightAmbient = vec4(0.0, 0.0, 0.0, 0.0);
vec4 lightDiffuse = vec4(1.0, 0.0, 0.0, 1.0);

const vec4 materialAmbient = vec4(0.0, 0.0, 0.0, 1.0);
const vec4 materialDiffuse = vec4(1.0, 1.0, 1.0, 1.0);
const vec4 materialSpecular = vec4(0.0, 0.0, 0.0, 0.0);
const float materialShininess = 20.0;

const vec4 sceneColor = vec4(0.0, 0.0, 0.0, 0.0);


void pointLight(in int i, in vec3 normal, in vec3 eye, in vec3 ecPosition3)
{
   float nDotVP;       // normal . light direction
   float nDotHV;       // normal . light half vector
   float pf;           // power factor
   float attenuation;  // computed attenuation factor
   float d;            // distance from surface to light source
   vec3  VP;           // direction from surface to light position
   vec3  halfVector;   // direction of maximum highlights

   // Compute vector from surface to light position
   VP = lightPosition - ecPosition3;

   // Compute distance between surface and light position
   d = length(VP);

   // Normalize the vector from surface to light position
   VP = normalize(VP);

   // Compute attenuation
   attenuation = 1.0 / (lightAttenuationConstant +
       lightAttenuationLinear * d +
       lightAttenuationQuadratic * d * d);

   halfVector = normalize(VP + eye);

   nDotVP = max(0.0, dot(normal, VP));
   nDotHV = max(0.0, dot(normal, halfVector));

   if (nDotVP == 0.0)
   {
       pf = 0.0;
   }
   else
   {
       pf = pow(nDotHV, materialShininess);

   }
   Ambient  += lightAmbient * attenuation;
   Diffuse  += lightDiffuse * nDotVP * attenuation;
//   Specular += lightSpecular * pf * attenuation;
}

vec3 fnormal(void)
{
    //Compute the normal 
    vec3 normal = gtf_Normal * gtf_NormalMatrix;
    normal = normalize(normal);
	
	// This should change to "return normal" but for this test, we force a normal pointing towards the light
	// return normal
    return vec3(0.0, 0.0, 1.0);
}

void flight(in vec3 normal, in vec4 ecPosition, float alphaFade)
{
    vec3 ecPosition3;
    vec3 eye;

    ecPosition3 = (vec3 (ecPosition)) / ecPosition.w;
    eye = vec3 (0.0, 0.0, 1.0);

    // Clear the light intensity accumulators
    Ambient  = vec4 (0.0);
    Diffuse  = vec4 (0.0);
    Specular = vec4 (0.0);
	
	lightDiffuse = gtf_Color;

    pointLight(0, normal, eye, ecPosition3);

    color = sceneColor +
      Ambient  * materialAmbient +
      Diffuse  * materialDiffuse;
    color += Specular * materialSpecular;
    color = clamp( color, 0.0, 1.0 );

    color.a *= alphaFade;
}


void main (void)
{
    vec3  transformedNormal;
    float alphaFade = 1.0;

    // Eye-coordinate position of vertex, needed in various calculations
    vec4 ecPosition = gtf_ModelViewMatrix * gtf_Vertex;

    // Do fixed functionality vertex transform
    gl_Position = gtf_ModelViewProjectionMatrix * gtf_Vertex;
    transformedNormal = fnormal();
    flight(transformedNormal, ecPosition, alphaFade);
}
