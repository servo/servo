/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests that you can not use bgra8unorm-srgb in compat mode.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('unsupportedTextureFormats')
  .desc(`Tests that you can not create a bgra8unorm-srgb texture in compat mode.`)
  .fn(t => {
    t.expectGPUError(
      'validation',
      () =>
        t.device.createTexture({
          size: [1, 1, 1],
          format: 'bgra8unorm-srgb',
          usage: GPUTextureUsage.TEXTURE_BINDING,
        }),
      true
    );
  });

g.test('unsupportedTextureViewFormats')
  .desc(
    `Tests that you can not create a bgra8unorm texture with a bgra8unorm-srgb viewFormat in compat mode.`
  )
  .fn(t => {
    t.expectGPUError(
      'validation',
      () =>
        t.device.createTexture({
          size: [1, 1, 1],
          format: 'bgra8unorm',
          viewFormats: ['bgra8unorm-srgb'],
          usage: GPUTextureUsage.TEXTURE_BINDING,
        }),
      true
    );
  });
