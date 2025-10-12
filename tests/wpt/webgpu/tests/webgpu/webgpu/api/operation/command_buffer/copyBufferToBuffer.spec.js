/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = 'copyBufferToBuffer operation tests';import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('single').
desc(
  `Validate the correctness of the copy by filling the srcBuffer with testable data, doing
  CopyBufferToBuffer() copy, and verifying the content of the whole dstBuffer with MapRead:
  Copy {4 bytes, part of, the whole} srcBuffer to the dstBuffer {with, without} a non-zero valid
  srcOffset that
  - covers the whole dstBuffer
  - covers the beginning of the dstBuffer
  - covers the end of the dstBuffer
  - covers neither the beginning nor the end of the dstBuffer`
).
params((u) =>
u //
// Whether the test case requires the newly-added method signature or not.
.combine('newSig', [false, true]).
beginSubcases().
combine('srcOffset', [0, 4, 8, 16, undefined]).
combine('dstOffset', [0, 4, 8, 16, undefined]).
unless(
  (p) => (p.srcOffset === undefined || p.dstOffset === undefined) && p.srcOffset !== p.dstOffset
).
combine('copySize', [0, 4, 8, 16, undefined]).
expand('srcBufferSize', (p) => [
(p.srcOffset ?? 0) + (p.copySize ?? 0),
(p.srcOffset ?? 0) + (p.copySize ?? 0) + 8]
).
expand('dstBufferSize', (p) => [
(p.dstOffset ?? 0) + (p.copySize ?? 0),
(p.dstOffset ?? 0) + (p.copySize ?? 0) + 8]
)
// Bifurcate the cases between newSig=false and newSig=true based on whether they need it.
.filter((p) => {
  const needsNewSignature = [p.srcOffset, p.dstOffset, p.copySize].includes(undefined);
  return p.newSig === needsNewSignature;
})
).
fn((t) => {
  const { srcOffset, dstOffset, copySize, srcBufferSize, dstBufferSize } = t.params;

  const srcData = new Uint8Array(srcBufferSize);
  for (let i = 0; i < srcBufferSize; ++i) {
    srcData[i] = i + 1;
  }

  const src = t.makeBufferWithContents(srcData, GPUBufferUsage.COPY_SRC);

  const dst = t.createBufferTracked({
    size: dstBufferSize,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  if (srcOffset === undefined || dstOffset === undefined) {
    assert(srcOffset === undefined && dstOffset === undefined);
    encoder.copyBufferToBuffer(src, dst, copySize);
  } else {
    encoder.copyBufferToBuffer(src, srcOffset, dst, dstOffset, copySize);
  }

  const expectedSrcOffset = srcOffset ?? 0;
  const expectedDstOffset = dstOffset ?? 0;
  const expectedCopySize = copySize ?? srcBufferSize - expectedSrcOffset;

  const isValid = dstBufferSize - expectedDstOffset >= expectedCopySize;
  t.expectValidationError(() => {
    t.device.queue.submit([encoder.finish()]);
  }, !isValid);

  const expectedDstData = new Uint8Array(dstBufferSize);
  for (let i = 0; i < expectedCopySize; ++i) {
    expectedDstData[expectedDstOffset + i] = srcData[expectedSrcOffset + i];
  }

  t.expectGPUBufferValuesEqual(dst, expectedDstData);
});

g.test('state_transitions').
desc(
  `Test proper state transitions/barriers happen between copy commands.
    Copy part of src to dst, then a different part of dst to src, and check contents of both.`
).
fn((t) => {
  const srcData = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
  const dstData = new Uint8Array([10, 20, 30, 40, 50, 60, 70, 80]);

  const src = t.makeBufferWithContents(
    srcData,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  const dst = t.makeBufferWithContents(
    dstData,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToBuffer(src, 0, dst, 4, 4);
  encoder.copyBufferToBuffer(dst, 0, src, 4, 4);
  t.device.queue.submit([encoder.finish()]);

  const expectedSrcData = new Uint8Array([1, 2, 3, 4, 10, 20, 30, 40]);
  const expectedDstData = new Uint8Array([10, 20, 30, 40, 1, 2, 3, 4]);
  t.expectGPUBufferValuesEqual(src, expectedSrcData);
  t.expectGPUBufferValuesEqual(dst, expectedDstData);
});

g.test('copy_order').
desc(
  `Test copy commands in one command buffer occur in the correct order.
    First copies one region from src to dst, then another region from src to an overlapping region
    of dst, then checks the dst buffer's contents.`
).
fn((t) => {
  const srcData = new Uint32Array([1, 2, 3, 4, 5, 6, 7, 8]);

  const src = t.makeBufferWithContents(srcData, GPUBufferUsage.COPY_SRC);

  const dst = t.createBufferTracked({
    size: srcData.length * 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToBuffer(src, 0, dst, 0, 16);
  encoder.copyBufferToBuffer(src, 16, dst, 8, 16);
  t.device.queue.submit([encoder.finish()]);

  const expectedDstData = new Uint32Array([1, 2, 5, 6, 7, 8, 0, 0]);
  t.expectGPUBufferValuesEqual(dst, expectedDstData);
});