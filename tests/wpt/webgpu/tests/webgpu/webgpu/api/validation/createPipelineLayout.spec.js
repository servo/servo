/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
createPipelineLayout validation tests.

TODO: review existing tests, write descriptions, and make sure tests are complete.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { bufferBindingTypeInfo, kBufferBindingTypes } from '../../capability_info.js';

import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = makeTestGroup(ValidationTest);

g.test('number_of_dynamic_buffers_exceeds_the_maximum_value').
desc(
  `
    Test that creating a pipeline layout fails with a validation error if the number of dynamic
    buffers exceeds the maximum value in the pipeline layout.
    - Test that creation of a pipeline using the maximum number of dynamic buffers added a dynamic
      buffer fails.

    TODO(#230): Update to enforce per-stage and per-pipeline-layout limits on BGLs as well.
  `
).
paramsSubcasesOnly((u) =>
u //
.combine('visibility', [0, 2, 4, 6]).
combine('type', kBufferBindingTypes)
).
fn((t) => {
  const { type, visibility } = t.params;
  const info = bufferBindingTypeInfo({ type });
  const { maxDynamicLimit } = info.perPipelineLimitClass;
  const perStageLimit = t.getDefaultLimit(info.perStageLimitClass.maxLimit);
  const maxDynamic = Math.min(
    maxDynamicLimit ? t.getDefaultLimit(maxDynamicLimit) : 0,
    perStageLimit
  );

  const maxDynamicBufferBindings = [];
  for (let binding = 0; binding < maxDynamic; binding++) {
    maxDynamicBufferBindings.push({
      binding,
      visibility,
      buffer: { type, hasDynamicOffset: true }
    });
  }

  const maxDynamicBufferBindGroupLayout = t.device.createBindGroupLayout({
    entries: maxDynamicBufferBindings
  });

  const goodDescriptor = {
    entries: [{ binding: 0, visibility, buffer: { type, hasDynamicOffset: false } }]
  };

  if (perStageLimit > maxDynamic) {
    const goodPipelineLayoutDescriptor = {
      bindGroupLayouts: [
      maxDynamicBufferBindGroupLayout,
      t.device.createBindGroupLayout(goodDescriptor)]

    };

    // Control case
    t.device.createPipelineLayout(goodPipelineLayoutDescriptor);
  }

  // Check dynamic buffers exceed maximum in pipeline layout.
  const badDescriptor = clone(goodDescriptor);
  badDescriptor.entries[0].buffer.hasDynamicOffset = true;

  const badPipelineLayoutDescriptor = {
    bindGroupLayouts: [
    maxDynamicBufferBindGroupLayout,
    t.device.createBindGroupLayout(badDescriptor)]

  };

  t.expectValidationError(() => {
    t.device.createPipelineLayout(badPipelineLayoutDescriptor);
  });
});

g.test('number_of_bind_group_layouts_exceeds_the_maximum_value').
desc(
  `
    Test that creating a pipeline layout fails with a validation error if the number of bind group
    layouts exceeds the maximum value in the pipeline layout.
    - Test that creation of a pipeline using the maximum number of bind groups added a bind group
      fails.
  `
).
fn((t) => {
  const bindGroupLayoutDescriptor = {
    entries: []
  };

  // 4 is the maximum number of bind group layouts.
  const maxBindGroupLayouts = [1, 2, 3, 4].map(() =>
  t.device.createBindGroupLayout(bindGroupLayoutDescriptor)
  );

  const goodPipelineLayoutDescriptor = {
    bindGroupLayouts: maxBindGroupLayouts
  };

  // Control case
  t.device.createPipelineLayout(goodPipelineLayoutDescriptor);

  // Check bind group layouts exceed maximum in pipeline layout.
  const badPipelineLayoutDescriptor = {
    bindGroupLayouts: [
    ...maxBindGroupLayouts,
    t.device.createBindGroupLayout(bindGroupLayoutDescriptor)]

  };

  t.expectValidationError(() => {
    t.device.createPipelineLayout(badPipelineLayoutDescriptor);
  });
});

g.test('bind_group_layouts,device_mismatch').
desc(
  `
    Tests createPipelineLayout cannot be called with bind group layouts created from another device
    Test with two layouts to make sure all layouts can be validated:
    - layout0 and layout1 from same device
    - layout0 and layout1 from different device
    `
).
paramsSubcasesOnly([
{ layout0Mismatched: false, layout1Mismatched: false }, // control case
{ layout0Mismatched: true, layout1Mismatched: false },
{ layout0Mismatched: false, layout1Mismatched: true }]
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { layout0Mismatched, layout1Mismatched } = t.params;

  const mismatched = layout0Mismatched || layout1Mismatched;

  const bglDescriptor = {
    entries: []
  };

  const layout0 = layout0Mismatched ?
  t.mismatchedDevice.createBindGroupLayout(bglDescriptor) :
  t.device.createBindGroupLayout(bglDescriptor);
  const layout1 = layout1Mismatched ?
  t.mismatchedDevice.createBindGroupLayout(bglDescriptor) :
  t.device.createBindGroupLayout(bglDescriptor);

  t.expectValidationError(() => {
    t.device.createPipelineLayout({ bindGroupLayouts: [layout0, layout1] });
  }, mismatched);
});