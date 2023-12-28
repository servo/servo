/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { GPUConst } from '../../../../constants.js';import { isPowerOfTwo } from '../../../../util/math.js';
import {
  kMinimumLimitBaseParams,
  makeLimitTestGroup } from


'./limit_utils.js';

function getDeviceLimitToRequest(
limitValueTest,
defaultLimit,
minimumLimit)
{
  switch (limitValueTest) {
    case 'atDefault':
      return defaultLimit;
    case 'overDefault':
      return 2 ** (Math.log2(defaultLimit) + 1);
    case 'betweenDefaultAndMinimum':
      return Math.min(
        minimumLimit,
        2 ** ((Math.log2(defaultLimit) + Math.log2(minimumLimit)) / 2 | 0)
      );
    case 'atMinimum':
      return minimumLimit;
    case 'underMinimum':
      return 2 ** (Math.log2(minimumLimit) - 1);
  }
}

function getTestValue(testValueName, requestedLimit) {
  switch (testValueName) {
    case 'atLimit':
      return requestedLimit;
    case 'underLimit':
      return 2 ** (Math.log2(requestedLimit) - 1);
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

const limit = 'minUniformBufferOffsetAlignment';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createBindGroup,at_over').
desc(`Test using createBindGroup at and over ${limit} limit`).
params(kMinimumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  // note: LimitTest.maximum is the adapter.limits[limit] value
  const { defaultLimit, adapterLimit: minimumLimit } = t;
  const { requestedLimit, testValue } = getDeviceLimitToRequestAndValueToTest(
    limitTest,
    testValueName,
    defaultLimit,
    minimumLimit
  );

  await t.testDeviceWithSpecificLimits(
    requestedLimit,
    testValue,
    async ({ device, testValue, shouldError }) => {
      const buffer = t.trackForCleanup(
        device.createBuffer({
          size: testValue * 2,
          usage: GPUBufferUsage.UNIFORM
        })
      );

      const layout = device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          buffer: {}
        }]

      });

      await t.expectValidationError(() => {
        device.createBindGroup({
          layout,
          entries: [
          {
            binding: 0,
            resource: {
              buffer,
              offset: testValue
            }
          }]

        });
      }, shouldError);
    }
  );
});

g.test('setBindGroup,at_over').
desc(`Test using setBindGroup at and over ${limit} limit`).
params(kMinimumLimitBaseParams).
fn(async (t) => {
  const { limitTest, testValueName } = t.params;
  // note: LimitTest.maximum is the adapter.limits[limit] value
  const { defaultLimit, adapterLimit: minimumLimit } = t;
  const { requestedLimit, testValue } = getDeviceLimitToRequestAndValueToTest(
    limitTest,
    testValueName,
    defaultLimit,
    minimumLimit
  );

  await t.testDeviceWithSpecificLimits(
    requestedLimit,
    testValue,
    async ({ device, testValue, shouldError }) => {
      const buffer = device.createBuffer({
        size: testValue * 2,
        usage: GPUBufferUsage.UNIFORM
      });

      const layout = device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUConst.ShaderStage.COMPUTE,
          buffer: {
            type: 'uniform',
            hasDynamicOffset: true
          }
        }]

      });

      const bindGroup = device.createBindGroup({
        layout,
        entries: [
        {
          binding: 0,
          resource: {
            buffer,
            size: testValue / 2
          }
        }]

      });

      const encoder = device.createCommandEncoder();
      const pass = encoder.beginComputePass();
      pass.setBindGroup(0, bindGroup, [testValue]);
      pass.end();

      await t.expectValidationError(() => {
        encoder.finish();
      }, shouldError);

      buffer.destroy();
    }
  );
});

g.test('validate,powerOf2').
desc('Verify that ${limit} is power of 2').
fn((t) => {
  t.expect(isPowerOfTwo(t.defaultLimit));
  t.expect(isPowerOfTwo(t.adapterLimit));
});

g.test('validate,greaterThanOrEqualTo32').
desc('Verify that ${limit} is >= 32').
fn((t) => {
  t.expect(t.defaultLimit >= 32);
  t.expect(t.adapterLimit >= 32);
});