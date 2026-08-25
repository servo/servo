/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';const kBufferParts = ['wholeBuffer', 'biggerBufferWithOffset'];


function getSizeAndOffsetForBufferPart(device, bufferPart, size) {
  const align = device.limits.minUniformBufferOffsetAlignment;
  switch (bufferPart) {
    case 'wholeBuffer':
      return { offset: 0, size };
    case 'biggerBufferWithOffset':
      return { size: size + align, offset: align };
  }
}

const limit = 'maxUniformBufferBindingSize';
export const { g, description } = makeLimitTestGroup(limit);

// We also need to update the maxBufferSize limit when testing.
const kExtraLimits = { maxBufferSize: 'maxLimit' };

g.test('createBindGroup,at_over').
desc(`Test using at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('bufferPart', kBufferParts)).
fn(async (t) => {
  const { limitTest, testValueName, bufferPart } = t.params;
  await t.testDeviceWithRequestedMaximumLimits(
    limitTest,
    testValueName,
    async ({ device, testValue, shouldError }) => {
      const bindGroupLayout = device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.VERTEX,
          buffer: {}
        }]

      });

      const { size, offset } = getSizeAndOffsetForBufferPart(device, bufferPart, testValue);

      // If the size of the buffer exceeds the related but separate maxBufferSize limit, we can
      // skip the validation since the allocation will fail with a validation error.
      if (size > device.limits.maxBufferSize) {
        return;
      }

      device.pushErrorScope('out-of-memory');
      const uniformBuffer = t.createBufferTracked({
        usage: GPUBufferUsage.UNIFORM,
        size
      });
      const outOfMemoryError = await device.popErrorScope();

      if (!outOfMemoryError) {
        await t.expectValidationError(
          () => {
            device.createBindGroup({
              layout: bindGroupLayout,
              entries: [
              {
                binding: 0,
                resource: {
                  buffer: uniformBuffer,
                  offset,
                  size: testValue
                }
              }]

            });
          },
          shouldError,
          `size: ${size}, offset: ${offset}, testValue: ${testValue}`
        );
      }
    },
    kExtraLimits
  );
});

g.test('validate,maxBufferSize').
desc(`Test that ${limit} <= maxBufferSize`).
fn((t) => {
  const { adapter, defaultLimit, adapterLimit } = t;
  t.expect(defaultLimit <= t.getDefaultLimit('maxBufferSize'));
  t.expect(adapterLimit <= adapter.limits.maxBufferSize);
});