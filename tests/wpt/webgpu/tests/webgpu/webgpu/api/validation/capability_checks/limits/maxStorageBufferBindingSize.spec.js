/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { align, roundDown } from '../../../../util/math.js';import {
  kMaximumLimitBaseParams,
  makeLimitTestGroup } from



'./limit_utils.js';

const kBufferParts = ['wholeBuffer', 'biggerBufferWithOffset'];


function getSizeAndOffsetForBufferPart(device, bufferPart, size) {
  const align = device.limits.minUniformBufferOffsetAlignment;
  switch (bufferPart) {
    case 'wholeBuffer':
      return { size, offset: 0 };
    case 'biggerBufferWithOffset':
      return { size: size + align, offset: align };
  }
}

const kStorageBufferRequiredSizeAlignment = 4;

// We also need to update the maxBufferSize limit when testing.
const kExtraLimits = { maxBufferSize: 'maxLimit' };

function getDeviceLimitToRequest(
limitValueTest,
defaultLimit,
maximumLimit)
{
  switch (limitValueTest) {
    case 'atDefault':
      return defaultLimit;
    case 'underDefault':
      return defaultLimit - kStorageBufferRequiredSizeAlignment;
    case 'betweenDefaultAndMaximum':
      return Math.floor((defaultLimit + maximumLimit) / 2);
    case 'atMaximum':
      return maximumLimit;
    case 'overMaximum':
      return maximumLimit + kStorageBufferRequiredSizeAlignment;
  }
}

function getTestValue(testValueName, requestedLimit) {
  switch (testValueName) {
    case 'atLimit':
      return roundDown(requestedLimit, kStorageBufferRequiredSizeAlignment);
    case 'overLimit':
      // Note: the requestedLimit might not meet alignment requirements.
      return align(
        requestedLimit + kStorageBufferRequiredSizeAlignment,
        kStorageBufferRequiredSizeAlignment
      );
  }
}

function getDeviceLimitToRequestAndValueToTest(
limitValueTest,
testValueName,
defaultLimit,
maximumLimit)
{
  const requestedLimit = getDeviceLimitToRequest(limitValueTest, defaultLimit, maximumLimit);
  return {
    requestedLimit,
    testValue: getTestValue(testValueName, requestedLimit)
  };
}

const limit = 'maxStorageBufferBindingSize';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBindGroup,at_over').
desc(`Test using createBindGroup at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('bufferPart', kBufferParts)).
fn(async (t) => {
  const { limitTest, testValueName, bufferPart } = t.params;
  const { defaultLimit, adapterLimit: maximumLimit } = t;
  const { requestedLimit, testValue } = getDeviceLimitToRequestAndValueToTest(
    limitTest,
    testValueName,
    defaultLimit,
    maximumLimit
  );

  await t.testDeviceWithSpecificLimits(
    requestedLimit,
    testValue,
    async ({ device, testValue, shouldError }) => {
      const bindGroupLayout = device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          buffer: { type: 'storage' }
        }]

      });

      const { size, offset } = getSizeAndOffsetForBufferPart(device, bufferPart, testValue);

      // If the size of the buffer exceeds the related but separate maxBufferSize limit, we can
      // skip the validation since the allocation will fail with a validation error.
      if (size > device.limits.maxBufferSize) {
        return;
      }

      device.pushErrorScope('out-of-memory');
      const storageBuffer = t.createBufferTracked({
        usage: GPUBufferUsage.STORAGE,
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
                  buffer: storageBuffer,
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

g.test('validate').
desc(`Test that ${limit} is a multiple of 4 bytes`).
fn((t) => {
  const { defaultLimit, adapterLimit } = t;
  t.expect(defaultLimit % 4 === 0);
  t.expect(adapterLimit % 4 === 0);
});

g.test('validate,maxBufferSize').
desc(`Test that ${limit} <= maxBufferSize`).
fn((t) => {
  const { adapter, defaultLimit, adapterLimit } = t;
  t.expect(defaultLimit <= t.getDefaultLimit('maxBufferSize'));
  t.expect(adapterLimit <= adapter.limits.maxBufferSize);
});