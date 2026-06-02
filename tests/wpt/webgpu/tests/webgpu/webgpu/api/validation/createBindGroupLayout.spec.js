/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
createBindGroupLayout validation tests.

TODO: make sure tests are complete.
`;import { AllFeaturesMaxLimitsGPUTest } from '../.././gpu_test.js';
import { kUnitCaseParamsBuilder } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import {
  kShaderStages,
  kShaderStageCombinations,
  kStorageTextureAccessValues,
  kTextureSampleTypes,
  kTextureViewDimensions,
  allBindingEntries,
  bindingTypeInfo,
  bufferBindingTypeInfo,
  kBufferBindingTypes,

  getBindingLimitForBindingType } from
'../../capability_info.js';
import {
  isTextureFormatUsableWithStorageAccessMode,
  kAllTextureFormats } from
'../../format_info.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

function isValidBufferTypeForStages(
device,
visibility,
type)
{
  if (type === 'read-only-storage' || type === 'storage') {
    if (visibility & GPUShaderStage.VERTEX) {
      if (!(device.limits.maxStorageBuffersInVertexStage > 0)) {
        return false;
      }
    }

    if (visibility & GPUShaderStage.FRAGMENT) {
      if (!(device.limits.maxStorageBuffersInFragmentStage > 0)) {
        return false;
      }
    }
  }

  return true;
}

function isValidStorageTextureForStages(device, visibility) {
  if (visibility & GPUShaderStage.VERTEX) {
    if (!(device.limits.maxStorageTexturesInVertexStage > 0)) {
      return false;
    }
  }

  if (visibility & GPUShaderStage.FRAGMENT) {
    if (!(device.limits.maxStorageTexturesInFragmentStage > 0)) {
      return false;
    }
  }

  return true;
}

function isValidBGLEntryForStages(device, visibility, entry) {
  return entry.storageTexture ?
  isValidStorageTextureForStages(device, visibility) :
  entry.buffer ?
  isValidBufferTypeForStages(device, visibility, entry.buffer?.type) :
  true;
}

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('duplicate_bindings').
desc('Test that uniqueness of binding numbers across entries is enforced.').
paramsSubcasesOnly([
{ bindings: [0, 1], _valid: true },
{ bindings: [0, 0], _valid: false }]
).
fn((t) => {
  const { bindings, _valid } = t.params;
  const entries = [];

  for (const binding of bindings) {
    entries.push({
      binding,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type: 'storage' }
    });
  }

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries
    });
  }, !_valid);
});

g.test('maximum_binding_limit').
desc(
  `
  Test that a validation error is generated if the binding number exceeds the maximum binding limit.

  TODO: Need to also test with higher limits enabled on the device, once we have a way to do that.
  `
).
paramsSubcasesOnly((u) =>
u.combine('bindingVariant', [1, 4, 8, 256, 'default', 'default-minus-one'])
).
fn((t) => {
  const { bindingVariant } = t.params;
  const entries = [];

  const binding =
  bindingVariant === 'default' ?
  t.device.limits.maxBindingsPerBindGroup :
  bindingVariant === 'default-minus-one' ?
  t.device.limits.maxBindingsPerBindGroup - 1 :
  bindingVariant;

  entries.push({
    binding,
    visibility: GPUShaderStage.COMPUTE,
    buffer: { type: 'storage' }
  });

  const success = binding < t.device.limits.maxBindingsPerBindGroup;

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries
    });
  }, !success);
});

g.test('visibility').
desc(
  `
    Test that only the appropriate combinations of visibilities are allowed for each resource type.
    - Test each possible combination of shader stage visibilities.
    - Test each type of bind group resource.`
).
params((u) =>
u.
combine('visibility', kShaderStageCombinations).
beginSubcases().
combine('entry', allBindingEntries(false))
).
fn((t) => {
  const { visibility, entry } = t.params;
  const info = bindingTypeInfo(entry);

  const success =
  (visibility & ~info.validStages) === 0 &&
  isValidBGLEntryForStages(t.device, visibility, entry);

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [{ binding: 0, visibility, ...entry }]
    });
  }, !success);
});

g.test('visibility,VERTEX_shader_stage_buffer_type').
desc(
  `
  Test that a validation error is generated if the buffer type is 'storage' when the
  visibility of the entry includes VERTEX.
  `
).
params((u) =>
u //
.combine('shaderStage', kShaderStageCombinations).
beginSubcases().
combine('type', kBufferBindingTypes)
).
fn((t) => {
  const { shaderStage, type } = t.params;

  const success =
  !(type === 'storage' && shaderStage & GPUShaderStage.VERTEX) &&
  isValidBufferTypeForStages(t.device, shaderStage, type);

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: shaderStage,
        buffer: { type }
      }]

    });
  }, !success);
});

g.test('visibility,VERTEX_shader_stage_storage_texture_access').
desc(
  `
  Test that a validation error is generated if the access value is 'write-only' when the
  visibility of the entry includes VERTEX.
  `
).
params((u) =>
u //
.combine('shaderStage', kShaderStageCombinations).
beginSubcases().
combine('access', [undefined, ...kStorageTextureAccessValues])
).
fn((t) => {
  const { shaderStage, access } = t.params;

  const appliedAccess = access ?? 'write-only';
  const success =
  !(
  // If visibility includes VERETX, storageTexture.access must be "read-only"
  shaderStage & GPUShaderStage.VERTEX && appliedAccess !== 'read-only') &&
  isValidStorageTextureForStages(t.device, shaderStage);

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: shaderStage,
        storageTexture: { access, format: 'r32uint' }
      }]

    });
  }, !success);
});

g.test('multisampled_validation').
desc(
  `
  Test that multisampling is only allowed if view dimensions is "2d" and the sampleType is not
  "float".
  `
).
params((u) =>
u //
.combine('viewDimension', [undefined, ...kTextureViewDimensions]).
beginSubcases().
combine('sampleType', [undefined, ...kTextureSampleTypes])
).
fn((t) => {
  const { viewDimension, sampleType } = t.params;

  const success =
  (viewDimension === '2d' || viewDimension === undefined) &&
  (sampleType ?? 'float') !== 'float';

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        texture: { multisampled: true, viewDimension, sampleType }
      }]

    });
  }, !success);
});

g.test('max_dynamic_buffers').
desc(
  `
    Test that limits on the maximum number of dynamic buffers are enforced.
    - Test creation of a bind group layout using the maximum number of dynamic buffers works.
    - Test creation of a bind group layout using the maximum number of dynamic buffers + 1 fails.
    - TODO(#230): Update to enforce per-stage and per-pipeline-layout limits on BGLs as well.`
).
params((u) =>
u.
combine('type', kBufferBindingTypes).
beginSubcases().
combine('extraDynamicBuffers', [0, 1]).
combine('staticBuffers', [0, 1])
).
fn((t) => {
  const { type, extraDynamicBuffers, staticBuffers } = t.params;
  const info = bufferBindingTypeInfo({ type });

  const limitName = info.perPipelineLimitClass.maxDynamicLimit;
  const bufferCount = limitName ? t.device.limits[limitName] : 0;
  const dynamicBufferCount = bufferCount + extraDynamicBuffers;
  const perStageLimit = t.device.limits[info.perStageLimitClass.maxLimits.COMPUTE];

  const entries = [];
  for (let i = 0; i < dynamicBufferCount; i++) {
    entries.push({
      binding: i,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type, hasDynamicOffset: true }
    });
  }

  for (let i = dynamicBufferCount; i < dynamicBufferCount + staticBuffers; i++) {
    entries.push({
      binding: i,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type, hasDynamicOffset: false }
    });
  }

  const descriptor = {
    entries
  };

  t.expectValidationError(
    () => {
      t.device.createBindGroupLayout(descriptor);
    },
    extraDynamicBuffers > 0 || entries.length > perStageLimit
  );
});

/**
 * One bind group layout will be filled with kPerStageBindingLimit[...] of the type |type|.
 * For each item in the array returned here, a case will be generated which tests a pipeline
 * layout with one extra bind group layout with one extra binding. That extra binding will have:
 *
 *   - If extraTypeSame, any of the binding types which counts toward the same limit as |type|.
 *     (i.e. 'storage-buffer' <-> 'readonly-storage-buffer').
 *   - Otherwise, an arbitrary other type.
 */
function* pickExtraBindingTypesForPerStage(entry, extraTypeSame) {
  if (extraTypeSame) {
    const info = bindingTypeInfo(entry);
    for (const extra of allBindingEntries(false)) {
      const extraInfo = bindingTypeInfo(extra);
      if (info.perStageLimitClass.class === extraInfo.perStageLimitClass.class) {
        yield extra;
      }
    }
  } else {
    yield entry.sampler ? { texture: {} } : { sampler: {} };
  }
}

const kMaxResourcesCases = kUnitCaseParamsBuilder.
combine('maxedEntry', allBindingEntries(false)).
beginSubcases().
combine('maxedVisibility', kShaderStages).
filter((p) => (bindingTypeInfo(p.maxedEntry).validStages & p.maxedVisibility) !== 0).
expand('extraEntry', (p) => [
...pickExtraBindingTypesForPerStage(p.maxedEntry, true),
...pickExtraBindingTypesForPerStage(p.maxedEntry, false)]
).
combine('extraVisibility', kShaderStages).
filter((p) => (bindingTypeInfo(p.extraEntry).validStages & p.extraVisibility) !== 0);

// Should never fail unless limitInfo.maxBindingsPerBindGroup.default is exceeded, because the validation for
// resources-of-type-per-stage is in pipeline layout creation.
g.test('max_resources_per_stage,in_bind_group_layout').
desc(
  `
    Test that the maximum number of bindings of a given type per-stage cannot be exceeded in a
    single bind group layout.
    - Test each binding type.
    - Test that creation of a bind group layout using the maximum number of bindings works.
    - Test that creation of a bind group layout using the maximum number of bindings + 1 fails.
    - TODO(#230): Update to enforce per-stage and per-pipeline-layout limits on BGLs as well.`
).
params(kMaxResourcesCases).
fn((t) => {
  const { maxedEntry, extraEntry, maxedVisibility, extraVisibility } = t.params;
  const maxedTypeInfo = bindingTypeInfo(maxedEntry);
  const maxedCount = getBindingLimitForBindingType(t.device, maxedVisibility, maxedEntry);
  const extraTypeInfo = bindingTypeInfo(extraEntry);

  t.skipIf(!isValidBGLEntryForStages(t.device, extraVisibility, extraEntry));

  const maxResourceBindings = [];
  for (let i = 0; i < maxedCount; i++) {
    maxResourceBindings.push({
      binding: i,
      visibility: maxedVisibility,
      ...maxedEntry
    });
  }

  const goodDescriptor = { entries: maxResourceBindings };

  // Control
  t.device.createBindGroupLayout(goodDescriptor);

  // Add an entry counting towards the same limit. It should produce a validation error.
  const newDescriptor = clone(goodDescriptor);
  newDescriptor.entries.push({
    binding: maxedCount,
    visibility: extraVisibility,
    ...extraEntry
  });

  const newBindingCountsTowardSamePerStageLimit =
  (maxedVisibility & extraVisibility) !== 0 &&
  maxedTypeInfo.perStageLimitClass.class === extraTypeInfo.perStageLimitClass.class;

  t.expectValidationError(() => {
    t.device.createBindGroupLayout(newDescriptor);
  }, newBindingCountsTowardSamePerStageLimit);
});

// One pipeline layout can have a maximum number of each type of binding *per stage* (which is
// different for each type). Test that the max works, then add one more binding of same-or-different
// type and same-or-different visibility.
g.test('max_resources_per_stage,in_pipeline_layout').
desc(
  `
    Test that the maximum number of bindings of a given type per-stage cannot be exceeded across
    multiple bind group layouts when creating a pipeline layout.
    - Test each binding type.
    - Test that creation of a pipeline using the maximum number of bindings works.
    - Test that creation of a pipeline using the maximum number of bindings + 1 fails.
  `
).
params(kMaxResourcesCases).
fn((t) => {
  const { maxedEntry, extraEntry, maxedVisibility, extraVisibility } = t.params;
  const maxedTypeInfo = bindingTypeInfo(maxedEntry);
  const maxedCount = getBindingLimitForBindingType(t.device, maxedVisibility, maxedEntry);
  const extraTypeInfo = bindingTypeInfo(extraEntry);

  t.skipIf(!isValidBGLEntryForStages(t.device, extraVisibility, extraEntry));

  const maxResourceBindings = [];
  for (let i = 0; i < maxedCount; i++) {
    maxResourceBindings.push({
      binding: i,
      visibility: maxedVisibility,
      ...maxedEntry
    });
  }

  const goodLayout = t.device.createBindGroupLayout({ entries: maxResourceBindings });

  // Control
  t.device.createPipelineLayout({ bindGroupLayouts: [goodLayout] });

  const extraLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: extraVisibility,
      ...extraEntry
    }]

  });

  // Some binding types use the same limit, e.g. 'storage-buffer' and 'readonly-storage-buffer'.
  const newBindingCountsTowardSamePerStageLimit =
  (maxedVisibility & extraVisibility) !== 0 &&
  maxedTypeInfo.perStageLimitClass.class === extraTypeInfo.perStageLimitClass.class;

  t.expectValidationError(() => {
    t.device.createPipelineLayout({ bindGroupLayouts: [goodLayout, extraLayout] });
  }, newBindingCountsTowardSamePerStageLimit);
});

g.test('storage_texture,layout_dimension').
desc(
  `
  Test that viewDimension is not cube or cube-array if storageTextureLayout is not undefined.
  `
).
params((u) =>
u //
.combine('viewDimension', [undefined, ...kTextureViewDimensions])
).
fn((t) => {
  const { viewDimension } = t.params;

  const success = viewDimension !== 'cube' && viewDimension !== `cube-array`;

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: { format: 'rgba8unorm', viewDimension }
      }]

    });
  }, !success);
});

g.test('storage_texture,formats').
desc(
  `
  Test that a validation error is generated if the format doesn't support the storage usage. A
  validation error is also generated if the format doesn't support the 'read-write' storage access
  when the storage access is 'read-write'.
  `
).
params((u) =>
u //
.combine('format', kAllTextureFormats) //
.combine('access', kStorageTextureAccessValues)
).
fn((t) => {
  const { format, access } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const success = isTextureFormatUsableWithStorageAccessMode(t.device, format, access);

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: { format, access }
      }]

    });
  }, !success);
});