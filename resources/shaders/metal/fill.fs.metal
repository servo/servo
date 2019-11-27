// Automatically generated from files in pathfinder/shaders/. Do not edit!
#include <metal_stdlib>
#include <simd/simd.h>

using namespace metal;

struct spvDescriptorSetBuffer0
{
    texture2d<float> uAreaLUT [[id(0)]];
    sampler uAreaLUTSmplr [[id(1)]];
};

struct main0_out
{
    float4 oFragColor [[color(0)]];
};

struct main0_in
{
    float2 vFrom [[user(locn0)]];
    float2 vTo [[user(locn1)]];
};

fragment main0_out main0(main0_in in [[stage_in]], constant spvDescriptorSetBuffer0& spvDescriptorSet0 [[buffer(0)]])
{
    main0_out out = {};
    float2 from = in.vFrom;
    float2 to = in.vTo;
    bool2 _29 = bool2(from.x < to.x);
    float2 left = float2(_29.x ? from.x : to.x, _29.y ? from.y : to.y);
    bool2 _39 = bool2(from.x < to.x);
    float2 right = float2(_39.x ? to.x : from.x, _39.y ? to.y : from.y);
    float2 window = fast::clamp(float2(from.x, to.x), float2(-0.5), float2(0.5));
    float offset = mix(window.x, window.y, 0.5) - left.x;
    float t = offset / (right.x - left.x);
    float y = mix(left.y, right.y, t);
    float d = (right.y - left.y) / (right.x - left.x);
    float dX = window.x - window.y;
    out.oFragColor = float4(spvDescriptorSet0.uAreaLUT.sample(spvDescriptorSet0.uAreaLUTSmplr, (float2(y + 8.0, abs(d * dX)) / float2(16.0))).x * dX);
    return out;
}

