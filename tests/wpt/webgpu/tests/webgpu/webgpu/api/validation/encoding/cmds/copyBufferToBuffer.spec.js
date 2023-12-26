/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
copyBufferToBuffer tests.

Test Plan:
* Buffer is valid/invalid
  - the source buffer is invalid
  - the destination buffer is invalid
* Buffer usages
  - the source buffer is created without GPUBufferUsage::COPY_SRC
  - the destination buffer is created without GPUBufferUsage::COPY_DEST
* CopySize
  - copySize is not a multiple of 4
  - copySize is 0
* copy offsets
  - sourceOffset is not a multiple of 4
  - destinationOffset is not a multiple of 4
* Arithmetic overflow
  - (sourceOffset + copySize) is overflow
  - (destinationOffset + copySize) is overflow
* Out of bounds
  - (sourceOffset + copySize) > size of source buffer
  - (destinationOffset + copySize) > size of destination buffer
* Source buffer and destination buffer are the same buffer
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kBufferUsages } from '../../../../capability_info.js';
import { kResourceStates } from '../../../../gpu_test.js';
import { kMaxSafeMultipleOf8 } from '../../../../util/math.js';
import { ValidationTest } from '../../validation_test.js';

class F extends ValidationTest {
  TestCopyBufferToBuffer(options)






  {
    const { srcBuffer, srcOffset, dstBuffer, dstOffset, copySize, expectation } = options;

    const commandEncoder = this.device.createCommandEncoder();
    commandEncoder.copyBufferToBuffer(srcBuffer, srcOffset, dstBuffer, dstOffset, copySize);

    if (expectation === 'FinishError') {
      this.expectValidationError(() => {
        commandEncoder.finish();
      });
    } else {
      const cmd = commandEncoder.finish();
      this.expectValidationError(() => {
        this.device.queue.submit([cmd]);
      }, expectation === 'SubmitError');
    }
  }
}

export const g = makeTestGroup(F);

g.test('buffer_state').
params((u) =>
u //
.combine('srcBufferState', kResourceStates).
combine('dstBufferState', kResourceStates)
).
fn((t) => {
  const { srcBufferState, dstBufferState } = t.params;
  const srcBuffer = t.createBufferWithState(srcBufferState, {
    size: 16,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const dstBuffer = t.createBufferWithState(dstBufferState, {
    size: 16,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const shouldFinishError = srcBufferState === 'invalid' || dstBufferState === 'invalid';
  const shouldSubmitSuccess = srcBufferState === 'valid' && dstBufferState === 'valid';
  const expectation = shouldSubmitSuccess ?
  'Success' :
  shouldFinishError ?
  'FinishError' :
  'SubmitError';

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset: 0,
    dstBuffer,
    dstOffset: 0,
    copySize: 8,
    expectation
  });
});

g.test('buffer,device_mismatch').
desc(
  'Tests copyBufferToBuffer cannot be called with src buffer or dst buffer created from another device'
).
paramsSubcasesOnly([
{ srcMismatched: false, dstMismatched: false }, // control case
{ srcMismatched: true, dstMismatched: false },
{ srcMismatched: false, dstMismatched: true }]
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { srcMismatched, dstMismatched } = t.params;

  const srcBufferDevice = srcMismatched ? t.mismatchedDevice : t.device;
  const srcBuffer = srcBufferDevice.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(srcBuffer);

  const dstBufferDevice = dstMismatched ? t.mismatchedDevice : t.device;
  const dstBuffer = dstBufferDevice.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_DST
  });
  t.trackForCleanup(dstBuffer);

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset: 0,
    dstBuffer,
    dstOffset: 0,
    copySize: 8,
    expectation: srcMismatched || dstMismatched ? 'FinishError' : 'Success'
  });
});

g.test('buffer_usage').
paramsSubcasesOnly((u) =>
u //
.combine('srcUsage', kBufferUsages).
combine('dstUsage', kBufferUsages)
).
fn((t) => {
  const { srcUsage, dstUsage } = t.params;

  const srcBuffer = t.device.createBuffer({
    size: 16,
    usage: srcUsage
  });
  const dstBuffer = t.device.createBuffer({
    size: 16,
    usage: dstUsage
  });

  const isSuccess = srcUsage === GPUBufferUsage.COPY_SRC && dstUsage === GPUBufferUsage.COPY_DST;
  const expectation = isSuccess ? 'Success' : 'FinishError';

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset: 0,
    dstBuffer,
    dstOffset: 0,
    copySize: 8,
    expectation
  });
});

g.test('copy_size_alignment').
paramsSubcasesOnly([
{ copySize: 0, _isSuccess: true },
{ copySize: 2, _isSuccess: false },
{ copySize: 4, _isSuccess: true },
{ copySize: 5, _isSuccess: false },
{ copySize: 8, _isSuccess: true }]
).
fn((t) => {
  const { copySize, _isSuccess: isSuccess } = t.params;

  const srcBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_SRC
  });
  const dstBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_DST
  });

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset: 0,
    dstBuffer,
    dstOffset: 0,
    copySize,
    expectation: isSuccess ? 'Success' : 'FinishError'
  });
});

g.test('copy_offset_alignment').
paramsSubcasesOnly([
{ srcOffset: 0, dstOffset: 0, _isSuccess: true },
{ srcOffset: 2, dstOffset: 0, _isSuccess: false },
{ srcOffset: 4, dstOffset: 0, _isSuccess: true },
{ srcOffset: 5, dstOffset: 0, _isSuccess: false },
{ srcOffset: 8, dstOffset: 0, _isSuccess: true },
{ srcOffset: 0, dstOffset: 2, _isSuccess: false },
{ srcOffset: 0, dstOffset: 4, _isSuccess: true },
{ srcOffset: 0, dstOffset: 5, _isSuccess: false },
{ srcOffset: 0, dstOffset: 8, _isSuccess: true },
{ srcOffset: 4, dstOffset: 4, _isSuccess: true }]
).
fn((t) => {
  const { srcOffset, dstOffset, _isSuccess: isSuccess } = t.params;

  const srcBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_SRC
  });
  const dstBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_DST
  });

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset,
    dstBuffer,
    dstOffset,
    copySize: 8,
    expectation: isSuccess ? 'Success' : 'FinishError'
  });
});

g.test('copy_overflow').
paramsSubcasesOnly([
{ srcOffset: 0, dstOffset: 0, copySize: kMaxSafeMultipleOf8 },
{ srcOffset: 16, dstOffset: 0, copySize: kMaxSafeMultipleOf8 },
{ srcOffset: 0, dstOffset: 16, copySize: kMaxSafeMultipleOf8 },
{ srcOffset: kMaxSafeMultipleOf8, dstOffset: 0, copySize: 16 },
{ srcOffset: 0, dstOffset: kMaxSafeMultipleOf8, copySize: 16 },
{ srcOffset: kMaxSafeMultipleOf8, dstOffset: 0, copySize: kMaxSafeMultipleOf8 },
{ srcOffset: 0, dstOffset: kMaxSafeMultipleOf8, copySize: kMaxSafeMultipleOf8 },
{
  srcOffset: kMaxSafeMultipleOf8,
  dstOffset: kMaxSafeMultipleOf8,
  copySize: kMaxSafeMultipleOf8
}]
).
fn((t) => {
  const { srcOffset, dstOffset, copySize } = t.params;

  const srcBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_SRC
  });
  const dstBuffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_DST
  });

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset,
    dstBuffer,
    dstOffset,
    copySize,
    expectation: 'FinishError'
  });
});

g.test('copy_out_of_bounds').
paramsSubcasesOnly([
{ srcOffset: 0, dstOffset: 0, copySize: 32, _isSuccess: true },
{ srcOffset: 0, dstOffset: 0, copySize: 36 },
{ srcOffset: 36, dstOffset: 0, copySize: 4 },
{ srcOffset: 0, dstOffset: 36, copySize: 4 },
{ srcOffset: 36, dstOffset: 0, copySize: 0 },
{ srcOffset: 0, dstOffset: 36, copySize: 0 },
{ srcOffset: 20, dstOffset: 0, copySize: 16 },
{ srcOffset: 20, dstOffset: 0, copySize: 12, _isSuccess: true },
{ srcOffset: 0, dstOffset: 20, copySize: 16 },
{ srcOffset: 0, dstOffset: 20, copySize: 12, _isSuccess: true }]
).
fn((t) => {
  const { srcOffset, dstOffset, copySize, _isSuccess = false } = t.params;

  const srcBuffer = t.device.createBuffer({
    size: 32,
    usage: GPUBufferUsage.COPY_SRC
  });
  const dstBuffer = t.device.createBuffer({
    size: 32,
    usage: GPUBufferUsage.COPY_DST
  });

  t.TestCopyBufferToBuffer({
    srcBuffer,
    srcOffset,
    dstBuffer,
    dstOffset,
    copySize,
    expectation: _isSuccess ? 'Success' : 'FinishError'
  });
});

g.test('copy_within_same_buffer').
paramsSubcasesOnly([
{ srcOffset: 0, dstOffset: 8, copySize: 4 },
{ srcOffset: 8, dstOffset: 0, copySize: 4 },
{ srcOffset: 0, dstOffset: 4, copySize: 8 },
{ srcOffset: 4, dstOffset: 0, copySize: 8 }]
).
fn((t) => {
  const { srcOffset, dstOffset, copySize } = t.params;

  const buffer = t.device.createBuffer({
    size: 16,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  t.TestCopyBufferToBuffer({
    srcBuffer: buffer,
    srcOffset,
    dstBuffer: buffer,
    dstOffset,
    copySize,
    expectation: 'FinishError'
  });
});