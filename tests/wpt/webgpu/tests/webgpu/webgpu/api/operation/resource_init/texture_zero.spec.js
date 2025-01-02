/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test uninitialized textures are initialized to zero when read.

TODO:
- test by sampling depth/stencil [1]
- test by copying out of stencil [2]
- test compressed texture formats [3]
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable } from '../../../../common/util/util.js';
import { kTextureFormatInfo } from '../../../format_info.js';

import { checkContentsByBufferCopy, checkContentsByTextureCopy } from './check_texture/by_copy.js';
import {
  checkContentsByDepthTest,
  checkContentsByStencilTest } from
'./check_texture/by_ds_test.js';
import { checkContentsBySampling } from './check_texture/by_sampling.js';
import {
  getRequiredTextureUsage,


  TextureZeroInitTest,
  kTestParams,
  UninitializeMethod,
  InitializedState } from
'./check_texture/texture_zero_init_test.js';

const checkContentsImpl = {
  Sample: checkContentsBySampling,
  CopyToBuffer: checkContentsByBufferCopy,
  CopyToTexture: checkContentsByTextureCopy,
  DepthTest: checkContentsByDepthTest,
  StencilTest: checkContentsByStencilTest,
  ColorBlending: (t) => t.skip('Not implemented'),
  Storage: (t) => t.skip('Not implemented')
};

export const g = makeTestGroup(TextureZeroInitTest);

g.test('uninitialized_texture_is_zero').
params(kTestParams).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(kTextureFormatInfo[t.params.format].feature);
}).
fn((t) => {
  const usage = getRequiredTextureUsage(
    t.params.format,
    t.params.sampleCount,
    t.params.uninitializeMethod,
    t.params.readMethod
  );

  const texture = t.createTextureTracked({
    size: [t.textureWidth, t.textureHeight, t.textureDepthOrArrayLayers],
    format: t.params.format,
    dimension: t.params.dimension,
    usage,
    mipLevelCount: t.params.mipLevelCount,
    sampleCount: t.params.sampleCount
  });

  if (t.params.canaryOnCreation) {
    // Initialize some subresources with canary values
    for (const subresourceRange of t.iterateInitializedSubresources()) {
      t.initializeTexture(texture, InitializedState.Canary, subresourceRange);
    }
  }

  switch (t.params.uninitializeMethod) {
    case UninitializeMethod.Creation:
      break;
    case UninitializeMethod.StoreOpClear:
      // Initialize the rest of the resources.
      for (const subresourceRange of t.iterateUninitializedSubresources()) {
        t.initializeTexture(texture, InitializedState.Canary, subresourceRange);
      }
      // Then use a store op to discard their contents.
      for (const subresourceRange of t.iterateUninitializedSubresources()) {
        t.discardTexture(texture, subresourceRange);
      }
      break;
    default:
      unreachable();
  }

  // Check that all uninitialized resources are zero.
  for (const subresourceRange of t.iterateUninitializedSubresources()) {
    checkContentsImpl[t.params.readMethod](
      t,
      t.params,
      texture,
      InitializedState.Zero,
      subresourceRange
    );
  }

  if (t.params.canaryOnCreation) {
    // Check the all other resources are unchanged.
    for (const subresourceRange of t.iterateInitializedSubresources()) {
      checkContentsImpl[t.params.readMethod](
        t,
        t.params,
        texture,
        InitializedState.Canary,
        subresourceRange
      );
    }
  }
});