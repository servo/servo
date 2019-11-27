// Automatically generated from files in pathfinder/shaders/. Do not edit!
#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    texture2d<float> uGammaLUT [[id(0)]];
    sampler uGammaLUTSmplr [[id(1)]];
    constant float4* uKernel [[id(2)]];
    texture2d<float> uSource [[id(3)]];
    sampler uSourceSmplr [[id(4)]];
    constant float2* uSourceSize [[id(5)]];
    constant int* uGammaCorrectionEnabled [[id(6)]];
    constant float4* uBGColor [[id(7)]];
    constant float4* uFGColor [[id(8)]];
};

struct main0_out
{
    float4 oFragColor [[color(0)]];
};

struct main0_in
{
    float2 vTexCoord [[user(locn0)]];
};

float sample1Tap(thread const float& offset, thread texture2d<float> uSource, thread const sampler uSourceSmplr, thread float2& vTexCoord)
{
    return uSource.sample(uSourceSmplr, float2(vTexCoord.x + offset, vTexCoord.y)).x;
}

void sample9Tap(thread float4& outAlphaLeft, thread float& outAlphaCenter, thread float4& outAlphaRight, thread const float& onePixel, thread float4 uKernel, thread texture2d<float> uSource, thread const sampler uSourceSmplr, thread float2& vTexCoord)
{
    float _89;
    if (uKernel.x > 0.0)
    {
        float param = (-4.0) * onePixel;
        _89 = sample1Tap(param, uSource, uSourceSmplr, vTexCoord);
    }
    else
    {
        _89 = 0.0;
    }
    float param_1 = (-3.0) * onePixel;
    float param_2 = (-2.0) * onePixel;
    float param_3 = (-1.0) * onePixel;
    outAlphaLeft = float4(_89, sample1Tap(param_1, uSource, uSourceSmplr, vTexCoord), sample1Tap(param_2, uSource, uSourceSmplr, vTexCoord), sample1Tap(param_3, uSource, uSourceSmplr, vTexCoord));
    float param_4 = 0.0;
    outAlphaCenter = sample1Tap(param_4, uSource, uSourceSmplr, vTexCoord);
    float param_5 = 1.0 * onePixel;
    float _120 = sample1Tap(param_5, uSource, uSourceSmplr, vTexCoord);
    float param_6 = 2.0 * onePixel;
    float _125 = sample1Tap(param_6, uSource, uSourceSmplr, vTexCoord);
    float param_7 = 3.0 * onePixel;
    float _130 = sample1Tap(param_7, uSource, uSourceSmplr, vTexCoord);
    float _134;
    if (uKernel.x > 0.0)
    {
        float param_8 = 4.0 * onePixel;
        _134 = sample1Tap(param_8, uSource, uSourceSmplr, vTexCoord);
    }
    else
    {
        _134 = 0.0;
    }
    outAlphaRight = float4(_120, _125, _130, _134);
}

float convolve7Tap(thread const float4& alpha0, thread const float3& alpha1, thread float4 uKernel)
{
    return dot(alpha0, uKernel) + dot(alpha1, uKernel.zyx);
}

float gammaCorrectChannel(thread const float& bgColor, thread const float& fgColor, thread texture2d<float> uGammaLUT, thread const sampler uGammaLUTSmplr)
{
    return uGammaLUT.sample(uGammaLUTSmplr, float2(fgColor, 1.0 - bgColor)).x;
}

float3 gammaCorrect(thread const float3& bgColor, thread const float3& fgColor, thread texture2d<float> uGammaLUT, thread const sampler uGammaLUTSmplr)
{
    float param = bgColor.x;
    float param_1 = fgColor.x;
    float param_2 = bgColor.y;
    float param_3 = fgColor.y;
    float param_4 = bgColor.z;
    float param_5 = fgColor.z;
    return float3(gammaCorrectChannel(param, param_1, uGammaLUT, uGammaLUTSmplr), gammaCorrectChannel(param_2, param_3, uGammaLUT, uGammaLUTSmplr), gammaCorrectChannel(param_4, param_5, uGammaLUT, uGammaLUTSmplr));
}

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    float3 alpha;
    if ((*spvDescriptorSet0.uKernel).w == 0.0)
    {
        alpha = spvDescriptorSet0.uSource.sample(spvDescriptorSet0.uSourceSmplr, in.vTexCoord).xxx;
    }
    else
    {
        float param_3 = 1.0 / (*spvDescriptorSet0.uSourceSize).x;
        float4 param;
        float param_1;
        float4 param_2;
        sample9Tap(param, param_1, param_2, param_3, (*spvDescriptorSet0.uKernel), spvDescriptorSet0.uSource, spvDescriptorSet0.uSourceSmplr, in.vTexCoord);
        float4 alphaLeft = param;
        float alphaCenter = param_1;
        float4 alphaRight = param_2;
        float4 param_4 = alphaLeft;
        float3 param_5 = float3(alphaCenter, alphaRight.xy);
        float r = convolve7Tap(param_4, param_5, (*spvDescriptorSet0.uKernel));
        float4 param_6 = float4(alphaLeft.yzw, alphaCenter);
        float3 param_7 = alphaRight.xyz;
        float g = convolve7Tap(param_6, param_7, (*spvDescriptorSet0.uKernel));
        float4 param_8 = float4(alphaLeft.zw, alphaCenter, alphaRight.x);
        float3 param_9 = alphaRight.yzw;
        float b = convolve7Tap(param_8, param_9, (*spvDescriptorSet0.uKernel));
        alpha = float3(r, g, b);
    }
    if ((*spvDescriptorSet0.uGammaCorrectionEnabled) != 0)
    {
        float3 param_10 = (*spvDescriptorSet0.uBGColor).xyz;
        float3 param_11 = alpha;
        alpha = gammaCorrect(param_10, param_11, spvDescriptorSet0.uGammaLUT, spvDescriptorSet0.uGammaLUTSmplr);
    }
    out.oFragColor = float4(mix((*spvDescriptorSet0.uBGColor).xyz, (*spvDescriptorSet0.uFGColor).xyz, alpha), 1.0);
    return out;
}

