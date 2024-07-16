/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for setVertexBuffer on render pass and render bundle.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { makeValueTestVariant } from '../../../../../../common/util/util.js';
import { GPUConst } from '../../../../../constants.js';
import { kResourceStates } from '../../../../../gpu_test.js';
import { ValidationTest } from '../../../validation_test.js';

import { kRenderEncodeTypeParams, buildBufferOffsetAndSizeOOBTestParams } from './render.js';

export const g = makeTestGroup(ValidationTest);

g.test('slot').
desc(
  `
Tests slot must be less than the maxVertexBuffers in device limits.
  `
).
paramsSubcasesOnly(
  kRenderEncodeTypeParams.combine('slotVariant', [
  { mult: 0, add: 0 },
  { mult: 1, add: -1 },
  { mult: 1, add: 0 }]
  )
).
fn((t) => {
  const { encoderType, slotVariant } = t.params;
  const maxVertexBuffers = t.device.limits.maxVertexBuffers;
  const slot = makeValueTestVariant(maxVertexBuffers, slotVariant);

  const vertexBuffer = t.createBufferWithState('valid', {
    size: 16,
    usage: GPUBufferUsage.VERTEX
  });

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(slot, vertexBuffer);
  validateFinish(slot < maxVertexBuffers);
});

g.test('vertex_buffer_state').
desc(
  `
Tests vertex buffer must be valid.
  `
).
paramsSubcasesOnly(kRenderEncodeTypeParams.combine('state', kResourceStates)).
fn((t) => {
  const { encoderType, state } = t.params;
  const vertexBuffer = t.createBufferWithState(state, {
    size: 16,
    usage: GPUBufferUsage.VERTEX
  });

  const { encoder, validateFinishAndSubmitGivenState } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(0, vertexBuffer);
  validateFinishAndSubmitGivenState(state);
});

g.test('vertex_buffer,device_mismatch').
desc('Tests setVertexBuffer cannot be called with a vertex buffer created from another device').
paramsSubcasesOnly(kRenderEncodeTypeParams.combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { encoderType, mismatched } = t.params;
  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const vertexBuffer = t.trackForCleanup(
    sourceDevice.createBuffer({
      size: 16,
      usage: GPUBufferUsage.VERTEX
    })
  );

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(0, vertexBuffer);
  validateFinish(!mismatched);
});

g.test('vertex_buffer_usage').
desc(
  `
Tests vertex buffer must have 'Vertex' usage.
  `
).
paramsSubcasesOnly(
  kRenderEncodeTypeParams.combine('usage', [
  GPUConst.BufferUsage.VERTEX, // control case
  GPUConst.BufferUsage.COPY_DST,
  GPUConst.BufferUsage.COPY_DST | GPUConst.BufferUsage.VERTEX]
  )
).
fn((t) => {
  const { encoderType, usage } = t.params;
  const vertexBuffer = t.createBufferTracked({
    size: 16,
    usage
  });

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(0, vertexBuffer);
  validateFinish((usage & GPUBufferUsage.VERTEX) !== 0);
});

g.test('offset_alignment').
desc(
  `
Tests offset must be a multiple of 4.
  `
).
paramsSubcasesOnly(kRenderEncodeTypeParams.combine('offset', [0, 2, 4])).
fn((t) => {
  const { encoderType, offset } = t.params;
  const vertexBuffer = t.createBufferTracked({
    size: 16,
    usage: GPUBufferUsage.VERTEX
  });

  const { encoder, validateFinish: finish } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(0, vertexBuffer, offset);
  finish(offset % 4 === 0);
});

g.test('offset_and_size_oob').
desc(
  `
Tests offset and size cannot be larger than vertex buffer size.
  `
).
paramsSubcasesOnly(buildBufferOffsetAndSizeOOBTestParams(4, 256)).
fn((t) => {
  const { encoderType, offset, size, _valid } = t.params;
  const vertexBuffer = t.createBufferTracked({
    size: 256,
    usage: GPUBufferUsage.VERTEX
  });

  const { encoder, validateFinish } = t.createEncoder(encoderType);
  encoder.setVertexBuffer(0, vertexBuffer, offset, size);
  validateFinish(_valid);
});