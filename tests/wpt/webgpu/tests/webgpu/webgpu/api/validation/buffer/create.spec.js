/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests for validation in createBuffer.
`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import {
  kAllBufferUsageBits,
  kBufferSizeAlignment,
  kBufferUsages,
  kLimitInfo,
} from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import { kMaxSafeMultipleOf8 } from '../../../util/math.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

assert(kBufferSizeAlignment === 4);
g.test('size')
  .desc(
    'Test buffer size alignment is validated to be a multiple of 4 if mappedAtCreation is true.'
  )
  .params(u =>
    u
      .combine('mappedAtCreation', [false, true])
      .beginSubcases()
      .combine('size', [
        0,
        kBufferSizeAlignment * 0.5,
        kBufferSizeAlignment,
        kBufferSizeAlignment * 1.5,
        kBufferSizeAlignment * 2,
      ])
  )
  .fn(t => {
    const { mappedAtCreation, size } = t.params;
    const isValid = !mappedAtCreation || size % kBufferSizeAlignment === 0;
    const usage = BufferUsage.COPY_SRC;
    t.expectGPUError(
      'validation',
      () => t.device.createBuffer({ size, usage, mappedAtCreation }),
      !isValid
    );
  });

g.test('limit')
  .desc('Test buffer size is validated against maxBufferSize.')
  .params(u =>
    u
      .beginSubcases()
      .combine('size', [
        kLimitInfo.maxBufferSize.default - 1,
        kLimitInfo.maxBufferSize.default,
        kLimitInfo.maxBufferSize.default + 1,
      ])
  )
  .fn(t => {
    const { size } = t.params;
    const isValid = size <= kLimitInfo.maxBufferSize.default;
    const usage = BufferUsage.COPY_SRC;
    t.expectGPUError('validation', () => t.device.createBuffer({ size, usage }), !isValid);
  });

const kInvalidUsage = 0x8000;
assert((kInvalidUsage & kAllBufferUsageBits) === 0);
g.test('usage')
  .desc('Test combinations of zero to two usage flags are validated to be valid.')
  .params(u =>
    u
      .combine('usage1', [0, ...kBufferUsages, kInvalidUsage])
      .combine('usage2', [0, ...kBufferUsages, kInvalidUsage])
      .beginSubcases()
      .combine('mappedAtCreation', [false, true])
  )
  .fn(t => {
    const { mappedAtCreation, usage1, usage2 } = t.params;
    const usage = usage1 | usage2;

    const isValid =
      usage !== 0 &&
      (usage & ~kAllBufferUsageBits) === 0 &&
      ((usage & GPUBufferUsage.MAP_READ) === 0 ||
        (usage & ~(GPUBufferUsage.COPY_DST | GPUBufferUsage.MAP_READ)) === 0) &&
      ((usage & GPUBufferUsage.MAP_WRITE) === 0 ||
        (usage & ~(GPUBufferUsage.COPY_SRC | GPUBufferUsage.MAP_WRITE)) === 0);

    t.expectGPUError(
      'validation',
      () => t.device.createBuffer({ size: kBufferSizeAlignment * 2, usage, mappedAtCreation }),
      !isValid
    );
  });

const BufferUsage = GPUConst.BufferUsage;

g.test('createBuffer_invalid_and_oom')
  .desc(
    `When creating a mappable buffer, it's expected that shmem may be immediately allocated
(in the content process, before validation occurs in the GPU process). If the buffer is really
large, though, it could fail shmem allocation before validation fails. Ensure that OOM error is
hidden behind the "more severe" validation error.`
  )
  .paramsSubcasesOnly(u =>
    u.combineWithParams([
      { _valid: true, usage: BufferUsage.UNIFORM, size: 16 },
      { _valid: true, usage: BufferUsage.STORAGE, size: 16 },
      // Invalid because UNIFORM is not allowed with map usages.
      { usage: BufferUsage.MAP_WRITE | BufferUsage.UNIFORM, size: 16 },
      { usage: BufferUsage.MAP_WRITE | BufferUsage.UNIFORM, size: kMaxSafeMultipleOf8 },
      { usage: BufferUsage.MAP_WRITE | BufferUsage.UNIFORM, size: 0x20_0000_0000 }, // 128 GiB
      { usage: BufferUsage.MAP_READ | BufferUsage.UNIFORM, size: 16 },
      { usage: BufferUsage.MAP_READ | BufferUsage.UNIFORM, size: kMaxSafeMultipleOf8 },
      { usage: BufferUsage.MAP_READ | BufferUsage.UNIFORM, size: 0x20_0000_0000 }, // 128 GiB
    ])
  )
  .fn(t => {
    const { _valid, usage, size } = t.params;

    t.expectGPUError('validation', () => t.device.createBuffer({ size, usage }), !_valid);
  });
