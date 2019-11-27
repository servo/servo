// Automatically generated from files in pathfinder/shaders/. Do not edit!
#pragma clang diagnostic ignored "-Wmissing-prototypes"

#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    constant float2* uTileSize [[id(0)]];
    constant float2* uStencilTextureSize [[id(1)]];
    constant float4x4* uTransform [[id(2)]];
    texture2d<float> uPaintTexture [[id(3)]];
    sampler uPaintTextureSmplr [[id(4)]];
};

struct main0_out
{
    float2 vTexCoord [[user(locn0)]];
    float vBackdrop [[user(locn1)]];
    float4 vColor [[user(locn2)]];
    float4 gl_Position [[position]];
};

struct main0_in
{
    uint2 aTessCoord [[attribute(0)]];
    uint3 aTileOrigin [[attribute(1)]];
    int aBackdrop [[attribute(2)]];
    int aTileIndex [[attribute(3)]];
    float2 aColorTexCoord [[attribute(4)]];
};

float2 computeTileOffset(thread const uint& tileIndex, thread const float& stencilTextureWidth, thread float2 uTileSize)
{
    uint tilesPerRow = uint(stencilTextureWidth / uTileSize.x);
    uint2 tileOffset = uint2(tileIndex % tilesPerRow, tileIndex / tilesPerRow);
    return float2(tileOffset) * uTileSize;
}

float4 getColor(thread texture2d<float> uPaintTexture, thread const sampler uPaintTextureSmplr, thread float2& aColorTexCoord)
{
    return uPaintTexture.sample(uPaintTextureSmplr, aColorTexCoord, level(0.0));
}

void computeVaryings(thread float2 uTileSize, thread uint3& aTileOrigin, thread uint2& aTessCoord, thread int& aTileIndex, thread float2 uStencilTextureSize, thread float2& vTexCoord, thread float& vBackdrop, thread int& aBackdrop, thread float4& vColor, thread float4& gl_Position, thread float4x4 uTransform, thread texture2d<float> uPaintTexture, thread const sampler uPaintTextureSmplr, thread float2& aColorTexCoord)
{
    float2 origin = float2(aTileOrigin.xy) + (float2(float(aTileOrigin.z & 15u), float(aTileOrigin.z >> 4u)) * 256.0);
    float2 position = (origin + float2(aTessCoord)) * uTileSize;
    uint param = uint(aTileIndex);
    float param_1 = uStencilTextureSize.x;
    float2 maskTexCoordOrigin = computeTileOffset(param, param_1, uTileSize);
    float2 maskTexCoord = maskTexCoordOrigin + (float2(aTessCoord) * uTileSize);
    vTexCoord = maskTexCoord / uStencilTextureSize;
    vBackdrop = float(aBackdrop);
    vColor = getColor(uPaintTexture, uPaintTextureSmplr, aColorTexCoord);
    gl_Position = uTransform * float4(position, 0.0, 1.0);
}

vertex main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    computeVaryings((*spvDescriptorSet0.uTileSize), in.aTileOrigin, in.aTessCoord, in.aTileIndex, (*spvDescriptorSet0.uStencilTextureSize), out.vTexCoord, out.vBackdrop, in.aBackdrop, out.vColor, out.gl_Position, (*spvDescriptorSet0.uTransform), spvDescriptorSet0.uPaintTexture, spvDescriptorSet0.uPaintTextureSmplr, in.aColorTexCoord);
    return out;
}

