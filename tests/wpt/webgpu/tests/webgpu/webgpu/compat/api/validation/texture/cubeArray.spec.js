/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests that you can not create cube array views in compat mode.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);
g.test('cube_array')
  .desc('Test you cannot create a cube array texture view.')
  .params(u => u.combine('dimension', ['cube', 'cube-array']))
  .fn(t => {
    const { dimension } = t.params;
    const texture = t.device.createTexture({
      size: [1, 1, 6],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING,
    });

    const isValid = dimension === 'cube';
    t.expectGPUError(
      'validation',
      () => texture.createView({ dimension, format: 'rgba8unorm' }),
      !isValid
    );
  });
