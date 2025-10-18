/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for multiDrawIndirect/multiDrawIndexedIndirect on render pass.
`;import { kUnitCaseParamsBuilder } from '../../../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import {
  GPUConst,
  kMaxUnsignedLongValue,
  kMaxUnsignedLongLongValue } from
'../../../../../constants.js';
import { kResourceStates, AllFeaturesMaxLimitsGPUTest } from '../../../../../gpu_test.js';
import * as vtu from '../../../validation_test_utils.js';

const kIndirectMultiDrawTestParams = kUnitCaseParamsBuilder.
combine('indexed', [true, false]).
combine('useDrawCountBuffer', [true, false]);

class F extends AllFeaturesMaxLimitsGPUTest {
  makeIndexBuffer() {
    return this.createBufferTracked({
      size: 16,
      usage: GPUBufferUsage.INDEX
    });
  }
}

export const g = makeTestGroup(F);

g.test('buffers_state').
desc(
  `
Tests indirect and draw count buffers must be valid.
  `
).
paramsSubcasesOnly(
  kIndirectMultiDrawTestParams.
  combine('indirectState', kResourceStates).
  combine('drawCountState', kResourceStates)
  // drawCountState only matters if useDrawCountBuffer=true
  .filter((p) => p.useDrawCountBuffer || p.drawCountState === 'valid')
  // Filter out a few unnecessary cases that would hit two errors in the same API call
  .filter(
    (p) =>
    p.indirectState === 'valid' ||
    p.drawCountState === 'valid' ||
    p.indirectState !== p.drawCountState
  )
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('chromium-experimental-multi-draw-indirect');
  const { indexed, indirectState, useDrawCountBuffer, drawCountState } = t.params;
  const indirectBuffer = vtu.createBufferWithState(t, indirectState, {
    size: 256,
    usage: GPUBufferUsage.INDIRECT
  });
  const drawCountBuffer = useDrawCountBuffer ?
  vtu.createBufferWithState(t, drawCountState, {
    size: 256,
    usage: GPUBufferUsage.INDIRECT
  }) :
  undefined;

  const { encoder, validateFinishAndSubmit } = t.createEncoder('render pass');
  encoder.setPipeline(vtu.createNoOpRenderPipeline(t));
  if (indexed) {
    encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');

    encoder.multiDrawIndexedIndirect(indirectBuffer, 0, 1, drawCountBuffer);
  } else {

    encoder.multiDrawIndirect(indirectBuffer, 0, 1, drawCountBuffer);
  }

  const shouldBeValid =
  indirectState !== 'invalid' && (!drawCountBuffer || drawCountState !== 'invalid');
  const submitShouldSucceedIfValid =
  indirectState !== 'destroyed' && (!drawCountBuffer || drawCountState !== 'destroyed');
  validateFinishAndSubmit(shouldBeValid, submitShouldSucceedIfValid);
});

g.test('buffers,device_mismatch').
desc(
  'Tests multiDraw(Indexed)Indirect cannot be called with buffers created from another device'
).
paramsSubcasesOnly(
  kIndirectMultiDrawTestParams.
  combineWithParams([
  { indirectMismatched: false, drawCountMismatched: false }, // control case
  { indirectMismatched: true, drawCountMismatched: false },
  { indirectMismatched: false, drawCountMismatched: true }]
  )
  // drawCountMismatched only matters if useDrawCountBuffer=true
  .filter((p) => p.useDrawCountBuffer || !p.drawCountMismatched)
).
beforeAllSubcases((t) => t.usesMismatchedDevice()).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('chromium-experimental-multi-draw-indirect');
  const { indexed, useDrawCountBuffer, indirectMismatched, drawCountMismatched } = t.params;

  const indirectDevice = indirectMismatched ? t.mismatchedDevice : t.device;
  const drawCountDevice = drawCountMismatched ? t.mismatchedDevice : t.device;

  const indirectBuffer = t.trackForCleanup(
    indirectDevice.createBuffer({
      size: 256,
      usage: GPUBufferUsage.INDIRECT
    })
  );
  const drawCountBuffer = useDrawCountBuffer ?
  t.trackForCleanup(
    drawCountDevice.createBuffer({
      size: 256,
      usage: GPUBufferUsage.INDIRECT
    })
  ) :
  undefined;

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(vtu.createNoOpRenderPipeline(t));
  if (indexed) {
    encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');

    encoder.multiDrawIndexedIndirect(indirectBuffer, 0, 1, drawCountBuffer);
  } else {

    encoder.multiDrawIndirect(indirectBuffer, 0, 1, drawCountBuffer);
  }
  validateFinish(!indirectMismatched && !drawCountMismatched);
});

g.test('indirect_buffer_usage').
desc(
  `
Tests indirect and draw count buffers must have 'Indirect' usage.
  `
).
paramsSubcasesOnly(
  kIndirectMultiDrawTestParams.
  combine('indirectUsage', [
  GPUConst.BufferUsage.INDIRECT,
  GPUConst.BufferUsage.VERTEX,
  GPUConst.BufferUsage.VERTEX | GPUConst.BufferUsage.INDIRECT]
  ).
  combine('drawCountUsage', [
  GPUConst.BufferUsage.INDIRECT,
  GPUConst.BufferUsage.VERTEX,
  GPUConst.BufferUsage.VERTEX | GPUConst.BufferUsage.INDIRECT]
  )
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('chromium-experimental-multi-draw-indirect');
  const { indexed, indirectUsage, useDrawCountBuffer, drawCountUsage } = t.params;

  const indirectBuffer = t.createBufferTracked({
    size: 256,
    usage: indirectUsage
  });
  const drawCountBuffer = useDrawCountBuffer ?
  t.createBufferTracked({
    size: 256,
    usage: drawCountUsage
  }) :
  undefined;

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(vtu.createNoOpRenderPipeline(t));
  if (indexed) {
    encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');

    encoder.multiDrawIndexedIndirect(indirectBuffer, 0, 1, drawCountBuffer);
  } else {

    encoder.multiDrawIndirect(indirectBuffer, 0, 1, drawCountBuffer);
  }
  const shouldSucceed =
  (indirectUsage & GPUBufferUsage.INDIRECT) !== 0 &&
  (!drawCountBuffer || drawCountUsage & GPUBufferUsage.INDIRECT) !== 0;
  validateFinish(shouldSucceed);
});

g.test('offsets_alignment').
desc(
  `
Tests indirect and draw count offsets must be a multiple of 4.
  `
).
paramsSubcasesOnly(
  kIndirectMultiDrawTestParams.combineWithParams([
  // Valid
  { indirectOffset: 0, drawCountOffset: 0 },
  { indirectOffset: 4, drawCountOffset: 0 },
  { indirectOffset: 0, drawCountOffset: 4 },
  // Invalid
  { indirectOffset: 2, drawCountOffset: 0 },
  { indirectOffset: 6, drawCountOffset: 0 },
  { indirectOffset: 0, drawCountOffset: 2 },
  { indirectOffset: 0, drawCountOffset: 6 }]
  )
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('chromium-experimental-multi-draw-indirect');
  const { indexed, indirectOffset, useDrawCountBuffer, drawCountOffset } = t.params;

  const indirectBuffer = t.createBufferTracked({
    size: 256,
    usage: GPUBufferUsage.INDIRECT
  });
  const drawCountBuffer = useDrawCountBuffer ?
  t.createBufferTracked({
    size: 256,
    usage: GPUBufferUsage.INDIRECT
  }) :
  undefined;

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(vtu.createNoOpRenderPipeline(t));
  if (indexed) {
    encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');

    encoder.multiDrawIndexedIndirect(
      indirectBuffer,
      indirectOffset,
      1,
      drawCountBuffer,
      drawCountOffset
    );
  } else {

    encoder.multiDrawIndirect(
      indirectBuffer,
      indirectOffset,
      1,
      drawCountBuffer,
      drawCountOffset
    );
  }

  // We need to figure out if https://github.com/gpuweb/gpuweb/pull/2315/files#r1773031950 applies.
  validateFinish(indirectOffset % 4 === 0 && (!useDrawCountBuffer || drawCountOffset % 4 === 0));
});

g.test('indirectBuffer_range').
desc(
  `
Tests multi indirect draw calls with various indirect offsets and buffer sizes without draw count buffer.
`
).
paramsSubcasesOnly((u) =>
u.
combine('indexed', [true, false]) //
.expandWithParams(function* (p) {
  const drawParamsSize = p.indexed ? 20 : 16;

  // Simple OOB test cases, using a delta from the exact required size
  for (const { maxDrawCount, offset } of [
  { maxDrawCount: 1, offset: 0 },
  { maxDrawCount: 1, offset: 4 },
  { maxDrawCount: 1, offset: drawParamsSize + 4 },
  { maxDrawCount: 2, offset: 0 },
  { maxDrawCount: 6, offset: drawParamsSize }])
  {
    const exactRequiredSize = offset + maxDrawCount * drawParamsSize;
    yield { offset, maxDrawCount, bufferSize: exactRequiredSize };
    yield { offset, maxDrawCount, bufferSize: exactRequiredSize - 1 };
  }

  // Additional test cases
  // - Buffer size is 0
  yield { offset: 0, maxDrawCount: 1, bufferSize: 0 };
  // - Buffer size is unaligned (OK)
  yield { offset: 0, maxDrawCount: 1, bufferSize: drawParamsSize + 1 };
  // - In-bounds, but non-multiple of 4 offset
  yield { offset: 2, maxDrawCount: 1, bufferSize: drawParamsSize + 8 };
  yield { offset: 6, maxDrawCount: 1, bufferSize: drawParamsSize + 8 };
  // - Out of bounds, (offset + (drawParamsSize * maxDrawCount)) may overflow
  yield { offset: kMaxUnsignedLongLongValue, maxDrawCount: 1, bufferSize: 1024 };
  yield { offset: 0, maxDrawCount: kMaxUnsignedLongValue, bufferSize: 1024 };
})
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('chromium-experimental-multi-draw-indirect');
  const { indexed, offset, maxDrawCount, bufferSize } = t.params;

  const indirectBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.INDIRECT
  });

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(vtu.createNoOpRenderPipeline(t));
  if (indexed) {
    encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');

    encoder.multiDrawIndexedIndirect(indirectBuffer, offset, maxDrawCount);
  } else {

    encoder.multiDrawIndirect(indirectBuffer, offset, maxDrawCount);
  }

  const paramsSize = indexed ? 20 : 16;
  const exactRequiredSize = offset + maxDrawCount * paramsSize;
  const valid = offset % 4 === 0 && bufferSize >= exactRequiredSize;
  validateFinish(valid);
});

g.test('drawCountBuffer_range').
desc(
  `
Tests multi indirect draw calls with various draw count offsets, and draw count buffer sizes.
  `
).
paramsSubcasesOnly((u) =>
u.
combine('indexed', [true, false]) //
.combineWithParams([
// In bounds
{ offset: 0, bufferSize: 4 },
{ offset: 0, bufferSize: 5 },
// In bounds, but non-multiple of 4 offset
{ offset: 2, bufferSize: 8 },
// Out of bounds, offset too big for drawCountBuffer
{ offset: 4, bufferSize: 7 },
// Out of bounds, (offset + kDrawCountSize) may overflow
{ offset: kMaxUnsignedLongLongValue, bufferSize: 1024 }]
)
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('chromium-experimental-multi-draw-indirect');
  const { indexed, bufferSize, offset } = t.params;

  const indirectBuffer = t.createBufferTracked({
    size: indexed ? 20 : 16,
    usage: GPUBufferUsage.INDIRECT
  });
  const drawCountBuffer = t.createBufferTracked({
    size: bufferSize,
    usage: GPUBufferUsage.INDIRECT
  });

  const { encoder, validateFinish } = t.createEncoder('render pass');
  encoder.setPipeline(vtu.createNoOpRenderPipeline(t));
  if (indexed) {
    encoder.setIndexBuffer(t.makeIndexBuffer(), 'uint32');

    encoder.multiDrawIndexedIndirect(indirectBuffer, 0, 1, drawCountBuffer, offset);
  } else {

    encoder.multiDrawIndirect(indirectBuffer, 0, 1, drawCountBuffer, offset);
  }

  const kDrawCountSize = 4;
  const valid = offset % 4 === 0 && offset + kDrawCountSize <= bufferSize;
  validateFinish(valid);
});