/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { keysOf } from '../../../../../common/util/data_tables.js';import { assert } from '../../../../../common/util/util.js';import { align, roundDown } from '../../../../util/math.js';

import {


  kMaximumLimitBaseParams,
  makeLimitTestGroup } from
'./limit_utils.js';

const limit = 'maxComputeWorkgroupStorageSize';
export const { g, description } = makeLimitTestGroup(limit);

// Each var is roundUp(16, SizeOf(T))
const kSmallestWorkgroupVarSize = 16;

const wgslF16Types = {
  f16: { alignOf: 2, sizeOf: 2, requireF16: true },
  'vec2<f16>': { alignOf: 4, sizeOf: 4, requireF16: true },
  'vec3<f16>': { alignOf: 8, sizeOf: 6, requireF16: true },
  'vec4<f16>': { alignOf: 8, sizeOf: 8, requireF16: true },
  'mat2x2<f16>': { alignOf: 4, sizeOf: 8, requireF16: true },
  'mat3x2<f16>': { alignOf: 4, sizeOf: 12, requireF16: true },
  'mat4x2<f16>': { alignOf: 4, sizeOf: 16, requireF16: true },
  'mat2x3<f16>': { alignOf: 8, sizeOf: 16, requireF16: true },
  'mat3x3<f16>': { alignOf: 8, sizeOf: 24, requireF16: true },
  'mat4x3<f16>': { alignOf: 8, sizeOf: 32, requireF16: true },
  'mat2x4<f16>': { alignOf: 8, sizeOf: 16, requireF16: true },
  'mat3x4<f16>': { alignOf: 8, sizeOf: 24, requireF16: true },
  'mat4x4<f16>': { alignOf: 8, sizeOf: 32, requireF16: true }
};

const wgslBaseTypes = {
  f32: { alignOf: 4, sizeOf: 4, requireF16: false },
  i32: { alignOf: 4, sizeOf: 4, requireF16: false },
  u32: { alignOf: 4, sizeOf: 4, requireF16: false },

  'vec2<f32>': { alignOf: 8, sizeOf: 8, requireF16: false },
  'vec2<i32>': { alignOf: 8, sizeOf: 8, requireF16: false },
  'vec2<u32>': { alignOf: 8, sizeOf: 8, requireF16: false },

  'vec3<f32>': { alignOf: 16, sizeOf: 12, requireF16: false },
  'vec3<i32>': { alignOf: 16, sizeOf: 12, requireF16: false },
  'vec3<u32>': { alignOf: 16, sizeOf: 12, requireF16: false },

  'vec4<f32>': { alignOf: 16, sizeOf: 16, requireF16: false },
  'vec4<i32>': { alignOf: 16, sizeOf: 16, requireF16: false },
  'vec4<u32>': { alignOf: 16, sizeOf: 16, requireF16: false },

  'mat2x2<f32>': { alignOf: 8, sizeOf: 16, requireF16: false },
  'mat3x2<f32>': { alignOf: 8, sizeOf: 24, requireF16: false },
  'mat4x2<f32>': { alignOf: 8, sizeOf: 32, requireF16: false },
  'mat2x3<f32>': { alignOf: 16, sizeOf: 32, requireF16: false },
  'mat3x3<f32>': { alignOf: 16, sizeOf: 48, requireF16: false },
  'mat4x3<f32>': { alignOf: 16, sizeOf: 64, requireF16: false },
  'mat2x4<f32>': { alignOf: 16, sizeOf: 32, requireF16: false },
  'mat3x4<f32>': { alignOf: 16, sizeOf: 48, requireF16: false },
  'mat4x4<f32>': { alignOf: 16, sizeOf: 64, requireF16: false },

  S1: { alignOf: 16, sizeOf: 48, requireF16: false },
  S2: { alignOf: 4, sizeOf: 16 * 7, requireF16: false },
  S3: { alignOf: 16, sizeOf: 32, requireF16: false }
};

const wgslTypes = { ...wgslF16Types, ...wgslBaseTypes };

const kWGSLTypes = keysOf(wgslTypes);

function getModuleForWorkgroupStorageSize(device, wgslType, size) {
  assert(size % kSmallestWorkgroupVarSize === 0);
  const { sizeOf, alignOf, requireF16 } = wgslTypes[wgslType];
  const unitSize = align(sizeOf, alignOf);
  const units = Math.floor(size / unitSize);
  const sizeUsed = align(units * unitSize, 16);
  const sizeLeft = size - sizeUsed;
  const extra = Math.floor(sizeLeft / kSmallestWorkgroupVarSize);

  const code =
  (requireF16 ? 'enable f16;\n' : '') +
  `
    struct S1 {
      a: f32,
      b: vec4f,
      c: u32,
    };
    struct S2 {
      a: array<vec3f, 7>,
    };
    struct S3 {
      a: vec3f,
      b: vec2f,
    };
    var<workgroup> d0: array<${wgslType}, ${units}>;
    ${extra ? `var<workgroup> d1: array<vec4<f32>, ${extra}>;` : ''}
    @compute @workgroup_size(1) fn main() {
      _ = d0;
      ${extra ? '_ = d1;' : ''}
    }
  `;
  return { module: device.createShaderModule({ code }), code };
}

function getDeviceLimitToRequest(
limitValueTest,
defaultLimit,
maximumLimit)
{
  switch (limitValueTest) {
    case 'atDefault':
      return defaultLimit;
    case 'underDefault':
      return defaultLimit - kSmallestWorkgroupVarSize;
    case 'betweenDefaultAndMaximum':
      return roundDown(Math.floor((defaultLimit + maximumLimit) / 2), kSmallestWorkgroupVarSize);
    case 'atMaximum':
      return maximumLimit;
    case 'overMaximum':
      return maximumLimit + kSmallestWorkgroupVarSize;
  }
}

function getTestValue(testValueName, requestedLimit) {
  switch (testValueName) {
    case 'atLimit':
      return requestedLimit;
    case 'overLimit':
      return requestedLimit + kSmallestWorkgroupVarSize;
  }
}

function getDeviceLimitToRequestAndValueToTest(
limitValueTest,
testValueName,
defaultLimit,
maximumLimit)
{
  const requestedLimit = getDeviceLimitToRequest(limitValueTest, defaultLimit, maximumLimit);
  const testValue = getTestValue(testValueName, requestedLimit);
  return {
    requestedLimit,
    testValue
  };
}

g.test('createComputePipeline,at_over').
desc(`Test using createComputePipeline(Async) at and over ${limit} limit`).
params(
  kMaximumLimitBaseParams.combine('async', [false, true]).combine('wgslType', kWGSLTypes)
).
fn(async (t) => {
  const { limitTest, testValueName, async, wgslType } = t.params;
  const { defaultLimit, adapterLimit: maximumLimit } = t;

  const hasF16 = t.adapter.features.has('shader-f16');
  if (!hasF16 && wgslType in wgslF16Types) {
    return;
  }

  const features = hasF16 ? ['shader-f16'] : [];

  const { requestedLimit, testValue } = getDeviceLimitToRequestAndValueToTest(
    limitTest,
    testValueName,
    defaultLimit,
    maximumLimit
  );
  await t.testDeviceWithSpecificLimits(
    requestedLimit,
    testValue,
    async ({ device, testValue, actualLimit, shouldError }) => {
      const { module, code } = getModuleForWorkgroupStorageSize(device, wgslType, testValue);

      await t.testCreatePipeline(
        'createComputePipeline',
        async,
        module,
        shouldError,
        `size: ${testValue}, limit: ${actualLimit}\n${code}`
      );
    },
    {},
    features
  );
});