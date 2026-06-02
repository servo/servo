/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for texture_utils.ts
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { assert } from '../../../../../../common/util/util.js';
import {
  isTextureFormatPossiblyMultisampled,
  kDepthStencilFormats } from
'../../../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import { getTextureDimensionFromView, virtualMipSize } from '../../../../../util/texture/base.js';
import {
  kTexelRepresentationInfo } from


'../../../../../util/texture/texel_data.js';
import { kShaderStages } from '../../../../validation/decl/util.js';

import {
  chooseTextureSize,
  convertPerTexelComponentToResultFormat,
  createTextureWithRandomDataAndGetTexels,
  graphWeights,
  makeRandomDepthComparisonTexelGenerator,
  queryMipLevelMixWeightsForDevice,
  readTextureToTexelViews,
  texelsApproximatelyEqual } from
'./texture_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

function texelFormat(texel, rep) {
  return rep.componentOrder.map((component) => `${component}: ${texel[component]}`).join(', ');
}

g.test('createTextureWithRandomDataAndGetTexels_with_generator').
desc(
  `
    Test createTextureWithRandomDataAndGetTexels with a generator. Generators
    are only used with textureXXXCompare builtins as we need specific random
    values to test these builtins with a depth reference value.
    `
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
combine('viewDimension', ['2d', '2d-array', 'cube', 'cube-array'])
).
fn(async (t) => {
  const { format, viewDimension } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureViewDimensionNotSupported(viewDimension);
  t.skipIfTextureFormatAndViewDimensionNotCompatible(format, viewDimension);
  // choose an odd size (9) so we're more likely to test alignment issue.
  const size = chooseTextureSize({ minSize: 9, minBlocks: 4, format, viewDimension });
  t.debug(`size: ${size.map((v) => v.toString()).join(', ')}`);
  const descriptor = {
    format,
    dimension: getTextureDimensionFromView(viewDimension),
    size,
    mipLevelCount: 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension })
  };
  await createTextureWithRandomDataAndGetTexels(t, descriptor, {
    generator: makeRandomDepthComparisonTexelGenerator(descriptor, 'equal')
  });
  // We don't expect any particular results. We just expect no validation errors.
});

g.test('readTextureToTexelViews').
desc('test readTextureToTexelViews for various formats and dimensions').
params((u) =>
u.
combineWithParams([
{ srcFormat: 'r8unorm', texelViewFormat: 'rgba32float' },
{ srcFormat: 'r8sint', texelViewFormat: 'rgba32sint' },
{ srcFormat: 'r8uint', texelViewFormat: 'rgba32uint' },
{ srcFormat: 'rgba32float', texelViewFormat: 'rgba32float' },
{ srcFormat: 'rgba32uint', texelViewFormat: 'rgba32uint' },
{ srcFormat: 'rgba32sint', texelViewFormat: 'rgba32sint' },
{ srcFormat: 'depth24plus', texelViewFormat: 'rgba32float' },
{ srcFormat: 'depth24plus-stencil8', texelViewFormat: 'rgba32float' },
{ srcFormat: 'stencil8', texelViewFormat: 'stencil8' }]
).
combine('viewDimension', ['1d', '2d', '2d-array', '3d', 'cube', 'cube-array']).
combine('sampleCount', [1, 4]).
unless(
  (t) =>
  t.sampleCount > 1 && (
  !isTextureFormatPossiblyMultisampled(t.srcFormat) || t.viewDimension !== '2d')
)
).
fn(async (t) => {
  const { srcFormat, texelViewFormat, viewDimension, sampleCount } = t.params;
  t.skipIfTextureViewDimensionNotSupported(viewDimension);
  t.skipIfTextureFormatAndViewDimensionNotCompatible(srcFormat, viewDimension);
  if (sampleCount > 1) {
    t.skipIfTextureFormatNotMultisampled(srcFormat);
  }
  // choose an odd size (9) so we're more likely to test alignment issue.
  const size = chooseTextureSize({ minSize: 9, minBlocks: 4, format: srcFormat, viewDimension });
  t.debug(`size: ${size.map((v) => v.toString()).join(', ')}`);
  const descriptor = {
    format: srcFormat,
    dimension: getTextureDimensionFromView(viewDimension),
    size,
    mipLevelCount: viewDimension === '1d' || sampleCount > 1 ? 1 : 3,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING,
    sampleCount,
    ...(t.isCompatibility && { textureBindingViewDimension: viewDimension })
  };
  const { texels: expectedTexelViews, texture } = await createTextureWithRandomDataAndGetTexels(
    t,
    descriptor
  );
  const actualTexelViews = await readTextureToTexelViews(t, texture, descriptor, texelViewFormat);

  assert(actualTexelViews.length === expectedTexelViews.length, 'num mip levels match');

  const errors = [];
  for (let mipLevel = 0; mipLevel < actualTexelViews.length; ++mipLevel) {
    const actualMipLevelTexelView = actualTexelViews[mipLevel];
    const expectedMipLevelTexelView = expectedTexelViews[mipLevel];
    const mipLevelSize = virtualMipSize(texture.dimension, size, mipLevel);

    const actualRep = kTexelRepresentationInfo[actualMipLevelTexelView.format];
    const expectedRep = kTexelRepresentationInfo[expectedMipLevelTexelView.format];

    for (let z = 0; z < mipLevelSize[2]; ++z) {
      for (let y = 0; y < mipLevelSize[1]; ++y) {
        for (let x = 0; x < mipLevelSize[0]; ++x) {
          for (let sampleIndex = 0; sampleIndex < sampleCount; ++sampleIndex) {
            const actual = actualMipLevelTexelView.color({ x, y, z, sampleIndex });
            const expected = expectedMipLevelTexelView.color({ x, y, z, sampleIndex });

            const actualRGBA = convertPerTexelComponentToResultFormat(
              actual,
              actualMipLevelTexelView.format
            );
            const expectedRGBA = convertPerTexelComponentToResultFormat(
              expected,
              expectedMipLevelTexelView.format
            );

            // This currently expects the exact same values in actual vs expected.
            // It's possible this needs to be relaxed slightly but only for non-integer formats.
            // For now, if the tests pass everywhere, we'll keep it at 0 tolerance.
            const maxFractionalDiff = 0;
            if (
            !texelsApproximatelyEqual(
              actualRGBA,
              actualMipLevelTexelView.format,
              expectedRGBA,
              expectedMipLevelTexelView.format,
              maxFractionalDiff
            ))
            {
              const actualStr = texelFormat(actual, actualRep);
              const expectedStr = texelFormat(expected, expectedRep);
              errors.push(
                `texel at ${x}, ${y}, ${z}, sampleIndex: ${sampleIndex} expected: ${expectedStr}, actual: ${actualStr}`
              );
            }
          }
        }
      }
    }

    assert(errors.length === 0, errors.join('\n'));
  }
});

function validateWeights(t, stage, builtin, weights) {
  const kNumMixSteps = weights.length - 1;
  const showWeights = () => `
${weights.map((v, i) => `${i.toString().padStart(2)}: ${v}`).join('\n')}

e = expected
A = actual
${graphWeights(32, weights)}
`;

  t.expect(
    weights[0] === 0,
    `stage: ${stage}, ${builtin}, weight 0 expected 0 but was ${weights[0]}\n${showWeights()}`
  );
  t.expect(
    weights[kNumMixSteps] === 1,
    `stage: ${stage}, ${builtin}, top weight expected 1 but was ${
    weights[kNumMixSteps]
    }\n${showWeights()}`
  );

  const dx = 1 / kNumMixSteps;
  for (let i = 0; i < kNumMixSteps; ++i) {
    const dy = weights[i + 1] - weights[i];
    // dy / dx because dy might be 0
    const slope = dy / dx;

    // Validate the slope is not going down.
    assert(
      slope >= 0,
      `stage: ${stage}, ${builtin}, weight[${i}] was not <= weight[${i + 1}]\n${showWeights()}`
    );

    // Validate the slope is not going up too steeply.
    // The correct slope is 1 / kNumMixSteps but Mac AMD and Mac Intel
    // have the wrong mix weights. 2 is enough to pass Mac AMD which we
    // decided is ok but will fail on Mac Intel in compute stage which we
    // decides is not ok.
    assert(
      slope <= 2,
      `stage: ${stage}, ${builtin}, slope from weight[${i}] to weight[${
      i + 1
      }] is > 2.\n${showWeights()}`
    );
  }

  // Test that we don't have a mostly flat set of weights.
  // Note: Ideally every value is unique but 66% is enough to pass AMD Mac
  // which we decided was ok but high enough to fail Intel Mac in a compute stage
  // which we decided is not ok.
  const kMinPercentUniqueWeights = 66;
  t.expect(
    new Set(weights).size >= (weights.length * kMinPercentUniqueWeights * 0.01 | 0),
    `stage: ${stage}, ${builtin}, expected at least ~${kMinPercentUniqueWeights}% unique weights\n${showWeights()}`
  );
}

g.test('weights').
desc(
  `
Test the mip level weights are linear.

Given 2 mip levels, textureSampleLevel(....., mipLevel) should return
mix(colorFromLevel0, colorFromLevel1, mipLevel).

Similarly, textureSampleGrad(...., ddx, ...) where ddx is
vec2(mix(1.0, 2.0, mipLevel) / textureWidth, 0) should so return
mix(colorFromLevel0, colorFromLevel1, mipLevel).

If we put 0,0,0,0 in level 0 and 1,1,1,1 in level 1 then we should arguably
be able to assert

    for (mipLevel = 0; mipLevel <= 1, mipLevel += 0.01) {
      assert(textureSampleLevel(t, s, vec2f(0.5), mipLevel) === mipLevel)
      ddx = vec2(mix(1.0, 2.0, mipLevel) / textureWidth, 0)
      assert(textureSampleGrad(t, s, vec2f(0.5), ddx, vec2f(0)) === mipLevel)
    }

Unfortunately, the GPUs do not do this. In particular:

AMD Mac goes like this: Not great but we allow it

 +----------------+
 |             ***|
 |           **   |
 |          *     |
 |        **      |
 |      **        |
 |     *          |
 |   **           |
 |***             |
 +----------------+

 Intel Mac goes like this in a compute stage

 +----------------+
 |         *******|
 |         *      |
 |        *       |
 |        *       |
 |       *        |
 |       *        |
 |      *         |
 |*******         |
 +----------------+

Where as they should go like this

 +----------------+
 |              **|
 |            **  |
 |          **    |
 |        **      |
 |      **        |
 |    **          |
 |  **            |
 |**              |
 +----------------+

To make the texture builtin tests pass, they use the mix weights we query from the GPU
even if they are arguably bad. This test is to surface the failure of the GPU
to use mix weights the approximate a linear interpolation.

We allow the AMD case as but disallow extreme Intel case. WebGPU implementations
are supposed to work around this issue by poly-filling on devices that fail this test.
`
).
params((u) => u.combine('stage', kShaderStages)).
fn(async (t) => {
  const { stage } = t.params;
  const weights = await queryMipLevelMixWeightsForDevice(t, t.params.stage);
  validateWeights(t, stage, 'textureSampleLevel', weights.sampleLevelWeights);
  validateWeights(t, stage, 'textureSampleGrad', weights.softwareMixToGPUMixGradWeights);
});