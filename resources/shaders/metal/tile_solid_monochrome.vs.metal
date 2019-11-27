// Automatically generated from files in pathfinder/shaders/. Do not edit!
#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    constant float2* uTileSize [[id(0)]];
    constant float4x4* uTransform [[id(1)]];
    constant float4* uColor [[id(2)]];
};

struct main0_out
{
    float4 vColor [[user(locn0)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    uint2 aTessCoord [[attribute(0)]];
    int2 aTileOrigin [[attribute(1)]];
};

float4 getColor(thread float4 uColor)
{
    return uColor;
}

void computeVaryings(thread int2& aTileOrigin, thread uint2& aTessCoord, thread float2 uTileSize, thread float4& vColor, thread float4& gl_Position, thread float4x4 uTransform, thread float4 uColor)
{
    float2 position = float2(aTileOrigin + int2(aTessCoord)) * uTileSize;
    vColor = getColor(uColor);
    gl_Position = uTransform * float4(position, 0.0, 1.0);
}

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    computeVaryings(in.aTileOrigin, in.aTessCoord, (*spvDescriptorSet0.uTileSize), out.vColor, out.gl_Position, (*spvDescriptorSet0.uTransform), (*spvDescriptorSet0.uColor));
    return out;
}

