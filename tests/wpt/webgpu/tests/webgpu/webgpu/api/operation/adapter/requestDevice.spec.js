/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test GPUAdapter.requestDevice.

Note tests explicitly destroy created devices so that tests don't have to wait for GC to clean up
potentially limited native resources.
`;import { Fixture } from '../../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { getGPU } from '../../../../common/util/navigator_gpu.js';
import { assert, assertReject, raceWithRejectOnTimeout } from '../../../../common/util/util.js';
import {
  getDefaultLimitsForAdapter,
  kFeatureNames,
  kLimits,
  kLimitClasses } from
'../../../capability_info.js';
import { clamp, isPowerOfTwo } from '../../../util/math.js';

export const g = makeTestGroup(Fixture);

g.test('default').
desc(
  `
    Test requesting the device with a variation of default parameters.
    - No features listed in default device
    - Default limits`
).
paramsSubcasesOnly((u) =>
u.combine('args', [
[],
[undefined],
[{}],
[{ requiredFeatures: [], requiredLimits: {} }]]
)
).
fn(async (t) => {
  const { args } = t.params;
  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);
  const device = await adapter.requestDevice(...args);
  assert(device !== null);

  // Default device should have no features.
  t.expect(device.features.size === 0, 'Default device should not have any features');
  // All limits should be defaults.
  const limitInfo = getDefaultLimitsForAdapter(adapter);
  for (const limit of kLimits) {
    t.expect(
      device.limits[limit] === limitInfo[limit].default,
      `Expected ${limit} == default: ${device.limits[limit]} != ${limitInfo[limit].default}`
    );
  }

  device.destroy();
});

g.test('invalid').
desc(
  `
    Test that requesting device on an invalid adapter resolves with lost device.
    - Induce invalid adapter via a device lost from a device.destroy()
    - Check the device is lost with reason 'destroyed'
    - Try creating another device on the now-stale adapter
    - Check that returns a device lost with 'unknown'
    `
).
fn(async (t) => {
  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  {
    // Request a device and destroy it immediately afterwards.
    const device = await adapter.requestDevice();
    assert(device !== null);
    device.destroy();
    const lostInfo = await device.lost;
    t.expect(lostInfo.reason === 'destroyed');
  }

  // The adapter should now be invalid since a device was lost. Requesting another device should
  // return an already lost device.
  const kTimeoutMS = 1000;
  const device = await adapter.requestDevice();
  const lost = await raceWithRejectOnTimeout(device.lost, kTimeoutMS, 'device was not lost');
  t.expect(lost.reason === 'unknown');
});

g.test('stale').
desc(
  `
    Test that adapter.requestDevice() can successfully return a device once, and once only.
    - Tests that we can successfully resolve after serial and concurrent rejections.
    - Tests that consecutive valid attempts only succeeds the first time, returning lost device otherwise.`
).
paramsSubcasesOnly((u) =>
u.
combine('initialError', [undefined, 'TypeError', 'OperationError']).
combine('awaitInitialError', [true, false]).
combine('awaitSuccess', [true, false]).
unless(
  ({ initialError, awaitInitialError }) => initialError === undefined && awaitInitialError
)
).
fn(async (t) => {
  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  const { initialError, awaitInitialError, awaitSuccess } = t.params;

  switch (initialError) {
    case undefined:
      break;
    case 'TypeError':
      // Cause a type error by requesting with an unknown feature.
      if (awaitInitialError) {
        await assertReject(
          'TypeError',
          adapter.requestDevice({ requiredFeatures: ['unknown-feature'] })
        );
      } else {
        t.shouldReject(
          'TypeError',
          adapter.requestDevice({ requiredFeatures: ['unknown-feature'] })
        );
      }
      break;
    case 'OperationError':
      // Cause an operation error by requesting with an alignment limit that is not a power of 2.
      if (awaitInitialError) {
        await assertReject(
          'OperationError',
          adapter.requestDevice({ requiredLimits: { minUniformBufferOffsetAlignment: 255 } })
        );
      } else {
        t.shouldReject(
          'OperationError',
          adapter.requestDevice({ requiredLimits: { minUniformBufferOffsetAlignment: 255 } })
        );
      }
      break;
  }

  let device = undefined;
  const promise = adapter.requestDevice();
  if (awaitSuccess) {
    device = await promise;
    assert(device !== null);
  } else {
    t.shouldResolve(
      (async () => {
        const device = await promise;
        device.destroy();
      })()
    );
  }

  const kTimeoutMS = 1000;
  const lostDevice = await adapter.requestDevice();
  const lost = await raceWithRejectOnTimeout(
    lostDevice.lost,
    kTimeoutMS,
    'adapter was not stale'
  );
  t.expect(lost.reason === 'unknown');

  // Make sure to destroy the valid device after trying to get a second one. Otherwise, the second
  // device may fail because the adapter is put into an invalid state from the destroy.
  if (device) {
    device.destroy();
  }
});

g.test('features,unknown').
desc(
  `
    Test requesting device with an unknown feature.`
).
fn(async (t) => {
  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  t.shouldReject(
    'TypeError',
    adapter.requestDevice({ requiredFeatures: ['unknown-feature'] })
  );
});

g.test('features,known').
desc(
  `
    Test requesting device with all features.
    - Succeeds with device supporting feature if adapter supports the feature.
    - Rejects if the adapter does not support the feature.`
).
params((u) => u.combine('feature', kFeatureNames)).
fn(async (t) => {
  const { feature } = t.params;

  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  const promise = adapter.requestDevice({ requiredFeatures: [feature] });
  if (adapter.features.has(feature)) {
    const device = await promise;
    t.expect(device.features.has(feature), 'Device should include the required feature');
  } else {
    t.shouldReject('TypeError', promise);
  }
});

g.test('limits,unknown').
desc(
  `
    Test that specifying limits that aren't part of the supported limit set causes
    requestDevice to reject.`
).
fn(async (t) => {
  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  const requiredLimits = { unknownLimitName: 9000 };

  t.shouldReject('OperationError', adapter.requestDevice({ requiredLimits }));
});

g.test('limits,supported').
desc(
  `
    Test that each supported limit can be specified with valid values.
    - Tests each limit with the default values given by the spec
    - Tests each limit with the supported values given by the adapter`
).
params((u) =>
u.combine('limit', kLimits).beginSubcases().combine('limitValue', ['default', 'adapter'])
).
fn(async (t) => {
  const { limit, limitValue } = t.params;

  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  const limitInfo = getDefaultLimitsForAdapter(adapter);
  let value = -1;
  switch (limitValue) {
    case 'default':
      value = limitInfo[limit].default;
      break;
    case 'adapter':
      value = adapter.limits[limit];
      break;
  }

  const device = await adapter.requestDevice({ requiredLimits: { [limit]: value } });
  assert(device !== null);
  t.expect(
    device.limits[limit] === value,
    'Devices reported limit should match the required limit'
  );
  device.destroy();
});

g.test('limit,better_than_supported').
desc(
  `
    Test that specifying a better limit than what the adapter supports causes requestDevice to
    reject.
    - Tests each limit
    - Tests requesting better limits by various amounts`
).
params((u) =>
u.
combine('limit', kLimits).
beginSubcases().
expandWithParams((p) => {
  switch (kLimitClasses[p.limit]) {
    case 'maximum':
      return [
      { mul: 1, add: 1 },
      { mul: 1, add: 100 }];

    case 'alignment':
      return [
      { mul: 1, add: -1 },
      { mul: 1 / 2, add: 0 },
      { mul: 1 / 1024, add: 0 }];

  }
})
).
fn(async (t) => {
  const { limit, mul, add } = t.params;

  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  const limitInfo = getDefaultLimitsForAdapter(adapter);
  const value = adapter.limits[limit] * mul + add;
  const requiredLimits = {
    [limit]: clamp(value, { min: 0, max: limitInfo[limit].maximumValue })
  };

  t.shouldReject('OperationError', adapter.requestDevice({ requiredLimits }));
});

g.test('limit,out_of_range').
desc(
  `
    Test that specifying limits that are out of range (<0, >MAX_SAFE_INTEGER, >2**31-2 for 32-bit
    limits, =0 for alignment limits) produce the appropriate error (TypeError or OperationError).
    `
).
params((u) =>
u.
combine('limit', kLimits).
beginSubcases().
expand('value', function* () {
  yield -(2 ** 64);
  yield Number.MIN_SAFE_INTEGER - 3;
  yield Number.MIN_SAFE_INTEGER - 1;
  yield Number.MIN_SAFE_INTEGER;
  yield -(2 ** 32);
  yield -1;
  yield 0;
  yield 2 ** 32 - 2;
  yield 2 ** 32 - 1;
  yield 2 ** 32;
  yield 2 ** 32 + 1;
  yield 2 ** 32 + 2;
  yield Number.MAX_SAFE_INTEGER;
  yield Number.MAX_SAFE_INTEGER + 1;
  yield Number.MAX_SAFE_INTEGER + 3;
  yield 2 ** 64;
  yield Number.MAX_VALUE;
})
).
fn(async (t) => {
  const { limit, value } = t.params;

  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);
  const limitInfo = getDefaultLimitsForAdapter(adapter)[limit];

  const requiredLimits = {
    [limit]: value
  };

  const errorName =
  value < 0 || value > Number.MAX_SAFE_INTEGER ?
  'TypeError' :
  limitInfo.class === 'maximum' && value > adapter.limits[limit] ?
  'OperationError' :
  limitInfo.class === 'alignment' && (value > 2 ** 31 || !isPowerOfTwo(value)) ?
  'OperationError' :
  false;

  if (errorName) {
    t.shouldReject(errorName, adapter.requestDevice({ requiredLimits }));
  } else {
    await adapter.requestDevice({ requiredLimits });
  }
});

g.test('limit,worse_than_default').
desc(
  `
    Test that specifying a worse limit than the default values required by the spec cause the value
    to clamp.
    - Tests each limit
    - Tests requesting worse limits by various amounts`
).
params((u) =>
u.
combine('limit', kLimits).
beginSubcases().
expandWithParams((p) => {
  switch (kLimitClasses[p.limit]) {
    case 'maximum':
      return [
      { mul: 1, add: -1 },
      { mul: 1, add: -100 }];

    case 'alignment':
      return [
      { mul: 1, add: 1 },
      { mul: 2, add: 0 },
      { mul: 1024, add: 0 }];

  }
})
).
fn(async (t) => {
  const { limit, mul, add } = t.params;

  const gpu = getGPU(t.rec);
  const adapter = await gpu.requestAdapter();
  assert(adapter !== null);

  const limitInfo = getDefaultLimitsForAdapter(adapter);
  const value = limitInfo[limit].default * mul + add;
  const requiredLimits = {
    [limit]: clamp(value, { min: 0, max: limitInfo[limit].maximumValue })
  };

  let success;
  switch (limitInfo[limit].class) {
    case 'alignment':
      success = isPowerOfTwo(value);
      break;
    case 'maximum':
      success = true;
      break;
  }

  if (success) {
    const device = await adapter.requestDevice({ requiredLimits });
    assert(device !== null);
    t.expect(
      device.limits[limit] === limitInfo[limit].default,
      'Devices reported limit should match the default limit'
    );
    device.destroy();
  } else {
    t.shouldReject('OperationError', adapter.requestDevice({ requiredLimits }));
  }
});