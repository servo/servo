/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';const limit = 'maxBufferSize';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBuffer,at_over').
desc(`Test using at and over ${limit} limit`).
params(kMaximumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, actualLimit, shouldError }) => {
      await t.testForValidationErrorWithPossibleOutOfMemoryError(
        () => {
          const buffer = device.createBuffer({
            usage: GPUBufferUsage.VERTEX,
            size: testValue
          });
          buffer.destroy();
        },
        shouldError,
        `size: ${testValue}, limit: ${actualLimit}`
      );
    }
  );
});