// Automatically generated from files in pathfinder/shaders/. Do not edit!
#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    constant float4* uGridlineColor [[id(0)]];
    constant float4* uGroundColor [[id(1)]];
};

struct main0_out
{
    float4 oFragColor [[color(0)]];
};

struct main0_in
{
    float2 vTexCoord [[user(locn0)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    float2 texCoordPx = fract(in.vTexCoord) / fwidth(in.vTexCoord);
    bool4 _33 = bool4(any(texCoordPx <= float2(1.0)));
    out.oFragColor = float4(_33.x ? (*spvDescriptorSet0.uGridlineColor).x : (*spvDescriptorSet0.uGroundColor).x, _33.y ? (*spvDescriptorSet0.uGridlineColor).y : (*spvDescriptorSet0.uGroundColor).y, _33.z ? (*spvDescriptorSet0.uGridlineColor).z : (*spvDescriptorSet0.uGroundColor).z, _33.w ? (*spvDescriptorSet0.uGridlineColor).w : (*spvDescriptorSet0.uGroundColor).w);
    return out;
}

