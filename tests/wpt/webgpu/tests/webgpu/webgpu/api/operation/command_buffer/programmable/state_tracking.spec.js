/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Ensure state is set correctly. Tries to stress state caching (setting different states multiple
times in different orders) for setBindGroup and setPipeline.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUConst } from '../../../../constants.js';
import { kProgrammableEncoderTypes } from '../../../../util/command_buffer_maker.js';

import { ProgrammableStateTest } from './programmable_state_test.js';

export const g = makeTestGroup(ProgrammableStateTest);

const kBufferUsage = GPUConst.BufferUsage.COPY_SRC | GPUConst.BufferUsage.STORAGE;

g.test('bind_group_indices').
desc(
  `
    Test that bind group indices can be declared in any order, regardless of their order in the shader.
    - Test places the value of buffer a - buffer b into the out buffer, then reads the result.
  `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes).
beginSubcases().
combine('groupIndices', [
{ a: 0, b: 1, out: 2 },
{ a: 1, b: 2, out: 0 },
{ a: 2, b: 0, out: 1 },
{ a: 0, b: 2, out: 1 },
{ a: 2, b: 1, out: 0 },
{ a: 1, b: 0, out: 2 }]
)
).
fn((t) => {
  const { encoderType, groupIndices } = t.params;

  const pipeline = t.createBindingStatePipeline(encoderType, groupIndices);

  const out = t.makeBufferWithContents(new Int32Array([0]), kBufferUsage);
  const bindGroups = {
    a: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([3]), kBufferUsage),
      'read-only-storage'
    ),
    b: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([2]), kBufferUsage),
      'read-only-storage'
    ),
    out: t.createBindGroup(out, 'storage')
  };

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);

  t.setPipeline(encoder, pipeline);
  encoder.setBindGroup(groupIndices.a, bindGroups.a);
  encoder.setBindGroup(groupIndices.b, bindGroups.b);
  encoder.setBindGroup(groupIndices.out, bindGroups.out);
  t.dispatchOrDraw(encoder);
  validateFinishAndSubmit(true, true);

  t.expectGPUBufferValuesEqual(out, new Int32Array([1]));
});

g.test('bind_group_order').
desc(
  `
    Test that the order in which you set the bind groups doesn't matter.
  `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes).
beginSubcases().
combine('setOrder', [
['a', 'b', 'out'],
['b', 'out', 'a'],
['out', 'a', 'b'],
['b', 'a', 'out'],
['a', 'out', 'b'],
['out', 'b', 'a']]
)
).
fn((t) => {
  const { encoderType, setOrder } = t.params;

  const groupIndices = { a: 0, b: 1, out: 2 };
  const pipeline = t.createBindingStatePipeline(encoderType, groupIndices);

  const out = t.makeBufferWithContents(new Int32Array([0]), kBufferUsage);
  const bindGroups = {
    a: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([3]), kBufferUsage),
      'read-only-storage'
    ),
    b: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([2]), kBufferUsage),
      'read-only-storage'
    ),
    out: t.createBindGroup(out, 'storage')
  };

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);
  t.setPipeline(encoder, pipeline);

  for (const bindingName of setOrder) {
    encoder.setBindGroup(groupIndices[bindingName], bindGroups[bindingName]);
  }

  t.dispatchOrDraw(encoder);
  validateFinishAndSubmit(true, true);

  t.expectGPUBufferValuesEqual(out, new Int32Array([1]));
});

g.test('bind_group_before_pipeline').
desc(
  `
    Test that setting bind groups prior to setting the pipeline is still valid.
  `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes).
beginSubcases().
combineWithParams([
{ setBefore: ['a', 'b'], setAfter: ['out'] },
{ setBefore: ['a'], setAfter: ['b', 'out'] },
{ setBefore: ['out', 'b'], setAfter: ['a'] },
{ setBefore: ['a', 'b', 'out'], setAfter: [] }]
)
).
fn((t) => {
  const { encoderType, setBefore, setAfter } = t.params;
  const groupIndices = { a: 0, b: 1, out: 2 };
  const pipeline = t.createBindingStatePipeline(encoderType, groupIndices);

  const out = t.makeBufferWithContents(new Int32Array([0]), kBufferUsage);
  const bindGroups = {
    a: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([3]), kBufferUsage),
      'read-only-storage'
    ),
    b: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([2]), kBufferUsage),
      'read-only-storage'
    ),
    out: t.createBindGroup(out, 'storage')
  };

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);

  for (const bindingName of setBefore) {
    encoder.setBindGroup(groupIndices[bindingName], bindGroups[bindingName]);
  }

  t.setPipeline(encoder, pipeline);

  for (const bindingName of setAfter) {
    encoder.setBindGroup(groupIndices[bindingName], bindGroups[bindingName]);
  }

  t.dispatchOrDraw(encoder);
  validateFinishAndSubmit(true, true);

  t.expectGPUBufferValuesEqual(out, new Int32Array([1]));
});

g.test('one_bind_group_multiple_slots').
desc(
  `
    Test that a single bind group may be bound to more than one slot.
  `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes)
).
fn((t) => {
  const { encoderType } = t.params;
  const pipeline = t.createBindingStatePipeline(encoderType, { a: 0, b: 1, out: 2 });

  const out = t.makeBufferWithContents(new Int32Array([1]), kBufferUsage);
  const bindGroups = {
    ab: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([3]), kBufferUsage),
      'read-only-storage'
    ),
    out: t.createBindGroup(out, 'storage')
  };

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);
  t.setPipeline(encoder, pipeline);

  encoder.setBindGroup(0, bindGroups.ab);
  encoder.setBindGroup(1, bindGroups.ab);
  encoder.setBindGroup(2, bindGroups.out);

  t.dispatchOrDraw(encoder);
  validateFinishAndSubmit(true, true);

  t.expectGPUBufferValuesEqual(out, new Int32Array([0]));
});

g.test('bind_group_multiple_sets').
desc(
  `
    Test that the last bind group set to a given slot is used when dispatching.
  `
).
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes)
).
fn((t) => {
  const { encoderType } = t.params;
  const pipeline = t.createBindingStatePipeline(encoderType, { a: 0, b: 1, out: 2 });

  const badOut = t.makeBufferWithContents(new Int32Array([-1]), kBufferUsage);
  const out = t.makeBufferWithContents(new Int32Array([0]), kBufferUsage);
  const bindGroups = {
    a: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([3]), kBufferUsage),
      'read-only-storage'
    ),
    b: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([2]), kBufferUsage),
      'read-only-storage'
    ),
    c: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([5]), kBufferUsage),
      'read-only-storage'
    ),
    badOut: t.createBindGroup(badOut, 'storage'),
    out: t.createBindGroup(out, 'storage')
  };

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);

  encoder.setBindGroup(1, bindGroups.c);

  t.setPipeline(encoder, pipeline);

  encoder.setBindGroup(0, bindGroups.c);
  encoder.setBindGroup(0, bindGroups.a);

  encoder.setBindGroup(2, bindGroups.badOut);

  encoder.setBindGroup(1, bindGroups.b);
  encoder.setBindGroup(2, bindGroups.out);

  t.dispatchOrDraw(encoder);
  validateFinishAndSubmit(true, true);

  t.expectGPUBufferValuesEqual(out, new Int32Array([1]));
  t.expectGPUBufferValuesEqual(badOut, new Int32Array([-1]));
});

g.test('compatible_pipelines').
desc('Test that bind groups can be shared between compatible pipelines.').
params((u) =>
u //
.combine('encoderType', kProgrammableEncoderTypes)
).
fn((t) => {
  const { encoderType } = t.params;
  const pipelineA = t.createBindingStatePipeline(encoderType, { a: 0, b: 1, out: 2 });
  const pipelineB = t.createBindingStatePipeline(
    encoderType,
    { a: 0, b: 1, out: 2 },
    'a.value + b.value'
  );

  const outA = t.makeBufferWithContents(new Int32Array([0]), kBufferUsage);
  const outB = t.makeBufferWithContents(new Int32Array([0]), kBufferUsage);
  const bindGroups = {
    a: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([3]), kBufferUsage),
      'read-only-storage'
    ),
    b: t.createBindGroup(
      t.makeBufferWithContents(new Int32Array([2]), kBufferUsage),
      'read-only-storage'
    ),
    outA: t.createBindGroup(outA, 'storage'),
    outB: t.createBindGroup(outB, 'storage')
  };

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType);
  encoder.setBindGroup(0, bindGroups.a);
  encoder.setBindGroup(1, bindGroups.b);

  t.setPipeline(encoder, pipelineA);
  encoder.setBindGroup(2, bindGroups.outA);
  t.dispatchOrDraw(encoder);

  t.setPipeline(encoder, pipelineB);
  encoder.setBindGroup(2, bindGroups.outB);
  t.dispatchOrDraw(encoder);

  validateFinishAndSubmit(true, true);

  t.expectGPUBufferValuesEqual(outA, new Int32Array([1]));
  t.expectGPUBufferValuesEqual(outB, new Int32Array([5]));
});