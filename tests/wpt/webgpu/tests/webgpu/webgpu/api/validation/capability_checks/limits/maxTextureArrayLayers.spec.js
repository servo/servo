/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';const limit = 'maxTextureArrayLayers';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createTexture,at_over').
desc(`Test using at and over ${limit} limit`).
params(kMaximumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      await t.testForValidationErrorWithPossibleOutOfMemoryError(() => {
        const texture = device.createTexture({
          size: [1, 1, testValue],
          format: 'rgba8unorm',
          usage: GPUTextureUsage.TEXTURE_BINDING
        });
        if (!shouldError) {
          texture.destroy();
        }
      }, shouldError);
    }
  );
});