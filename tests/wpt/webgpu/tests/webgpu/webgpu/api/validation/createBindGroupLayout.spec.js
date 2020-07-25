/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
createBindGroupLayout validation tests.
`;
import { pbool, poptions, params } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import {
  kBindingTypeInfo,
  kBindingTypes,
  kBufferBindingTypeInfo,
  kMaxBindingsPerBindGroup,
  kShaderStages,
  kShaderStageCombinations,
  kTextureBindingTypeInfo,
  kTextureComponentTypes,
  kTextureViewDimensions,
} from '../../capability_info.js';

import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = makeTestGroup(ValidationTest);

g.test('some_binding_index_was_specified_more_than_once').fn(async t => {
  const goodDescriptor = {
    entries: [
      { binding: 0, visibility: GPUShaderStage.COMPUTE, type: 'storage-buffer' },
      { binding: 1, visibility: GPUShaderStage.COMPUTE, type: 'storage-buffer' },
    ],
  };

  // Control case
  t.device.createBindGroupLayout(goodDescriptor);

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.entries[1].binding = 0;

  // Binding index 0 can't be specified twice.
  t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
});

g.test('visibility_and_dynamic_offsets')
  .params(
    params()
      .combine(poptions('type', kBindingTypes))
      .combine(pbool('hasDynamicOffset'))
      .combine(poptions('visibility', kShaderStageCombinations))
  )
  .fn(async t => {
    const { type, hasDynamicOffset, visibility } = t.params;
    const info = kBindingTypeInfo[type];

    const supportsDynamicOffset = kBindingTypeInfo[type].perPipelineLimitClass.maxDynamic > 0;
    let success = true;
    if (!supportsDynamicOffset && hasDynamicOffset) success = false;
    if ((visibility & ~info.validStages) !== 0) success = false;

    // When hasDynamicOffset is false, it actually tests visibility.
    t.expectValidationError(() => {
      t.device.createBindGroupLayout({
        entries: [{ binding: 0, visibility, type, hasDynamicOffset }],
      });
    }, !success);
  });

g.test('min_buffer_binding_size')
  .params(
    params()
      .combine(poptions('type', kBindingTypes))
      .combine(poptions('minBufferBindingSize', [undefined, 0, 4]))
  )
  .fn(async t => {
    const { type, minBufferBindingSize } = t.params;

    let success = false;
    if (
      minBufferBindingSize === undefined ||
      minBufferBindingSize === 0 ||
      type in kBufferBindingTypeInfo
    ) {
      success = true;
    }

    t.expectValidationError(() => {
      t.device.createBindGroupLayout({
        entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, type, minBufferBindingSize }],
      });
    }, !success);
  });

g.test('view_dimension')
  .params(
    params()
      .combine(poptions('type', kBindingTypes))
      .combine(poptions('viewDimension', [undefined, ...kTextureViewDimensions]))
  )
  .fn(async t => {
    const { type, viewDimension } = t.params;

    const success = viewDimension === undefined || type in kTextureBindingTypeInfo;

    t.expectValidationError(() => {
      t.device.createBindGroupLayout({
        entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, type, viewDimension }],
      });
    }, !success);
  });

g.test('texture_component_type')
  .params(
    params()
      .combine(poptions('type', kBindingTypes))
      .combine(poptions('textureComponentType', [undefined, ...kTextureComponentTypes]))
  )
  .fn(async t => {
    const { type, textureComponentType } = t.params;

    const success =
      textureComponentType === undefined || kBindingTypeInfo[type].resource === 'sampledTex';

    t.expectValidationError(() => {
      t.device.createBindGroupLayout({
        entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, type, textureComponentType }],
      });
    }, !success);
  });

g.test('multisampled')
  .params(
    params()
      .combine(poptions('type', kBindingTypes))
      .combine(poptions('multisampled', [undefined, false, true]))
  )
  .fn(async t => {
    const { type, multisampled } = t.params;

    const success = multisampled === false || kBindingTypeInfo[type].resource === 'sampledTex';

    t.expectValidationError(() => {
      t.device.createBindGroupLayout({
        entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, type, multisampled }],
      });
    }, !success);
  });

g.test('storage_texture_format')
  .params(
    params()
      .combine(poptions('type', kBindingTypes))
      .combine(poptions('storageTextureFormat', [undefined, 'rgba8unorm']))
  )
  .fn(async t => {
    const { type, storageTextureFormat } = t.params;

    const success =
      storageTextureFormat === undefined || kBindingTypeInfo[type].resource === 'storageTex';

    t.expectValidationError(() => {
      t.device.createBindGroupLayout({
        entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, type, storageTextureFormat }],
      });
    }, !success);
  });

g.test('number_of_dynamic_buffers_exceeds_the_maximum_value')
  .params([
    { type: 'storage-buffer', maxDynamicBufferCount: 4 },
    { type: 'uniform-buffer', maxDynamicBufferCount: 8 },
  ])
  .fn(async t => {
    const { type, maxDynamicBufferCount } = t.params;

    const maxDynamicBufferBindings = [];
    for (let i = 0; i < maxDynamicBufferCount; i++) {
      maxDynamicBufferBindings.push({
        binding: i,
        visibility: GPUShaderStage.COMPUTE,
        type,
        hasDynamicOffset: true,
      });
    }

    const goodDescriptor = {
      entries: [
        ...maxDynamicBufferBindings,
        {
          binding: maxDynamicBufferBindings.length,
          visibility: GPUShaderStage.COMPUTE,
          type,
          hasDynamicOffset: false,
        },
      ],
    };

    // Control case
    t.device.createBindGroupLayout(goodDescriptor);

    // Dynamic buffers exceed maximum in a bind group layout.
    const badDescriptor = clone(goodDescriptor);
    badDescriptor.entries[maxDynamicBufferCount].hasDynamicOffset = true;

    t.expectValidationError(() => {
      t.device.createBindGroupLayout(badDescriptor);
    });
  });

// One bind group layout will be filled with kPerStageBindingLimit[...] of the type |type|.
// For each item in the array returned here, a case will be generated which tests a pipeline
// layout with one extra bind group layout with one extra binding. That extra binding will have:
//
//   - If extraTypeSame, any of the binding types which counts toward the same limit as |type|.
//     (i.e. 'storage-buffer' <-> 'readonly-storage-buffer').
//   - Otherwise, an arbitrary other type.
function* pickExtraBindingTypes(bindingType, extraTypeSame) {
  const info = kBindingTypeInfo[bindingType];
  if (extraTypeSame) {
    for (const extraBindingType of kBindingTypes) {
      if (
        info.perStageLimitClass.class ===
        kBindingTypeInfo[extraBindingType].perStageLimitClass.class
      ) {
        yield extraBindingType;
      }
    }
  } else {
    yield info.perStageLimitClass.class === 'sampler' ? 'sampled-texture' : 'sampler';
  }
}

const kCasesForMaxResourcesPerStageTests = params()
  .combine(poptions('maxedType', kBindingTypes))
  .combine(poptions('maxedVisibility', kShaderStages))
  .filter(p => (kBindingTypeInfo[p.maxedType].validStages & p.maxedVisibility) !== 0)
  .expand(function* (p) {
    for (const extraTypeSame of [true, false]) {
      yield* poptions('extraType', pickExtraBindingTypes(p.maxedType, extraTypeSame));
    }
  })
  .combine(poptions('extraVisibility', kShaderStages))
  .filter(p => (kBindingTypeInfo[p.extraType].validStages & p.extraVisibility) !== 0);

// Should never fail unless kMaxBindingsPerBindGroup is exceeded, because the validation for
// resources-of-type-per-stage is in pipeline layout creation.
g.test('max_resources_per_stage,in_bind_group_layout')
  .params(kCasesForMaxResourcesPerStageTests)
  .fn(async t => {
    const { maxedType, extraType, maxedVisibility, extraVisibility } = t.params;
    const maxedTypeInfo = kBindingTypeInfo[maxedType];
    const maxedCount = maxedTypeInfo.perStageLimitClass.max;
    const extraTypeInfo = kBindingTypeInfo[extraType];

    const maxResourceBindings = [];
    for (let i = 0; i < maxedCount; i++) {
      maxResourceBindings.push({
        binding: i,
        visibility: maxedVisibility,
        type: maxedType,
        storageTextureFormat: maxedTypeInfo.resource === 'storageTex' ? 'rgba8unorm' : undefined,
      });
    }

    const goodDescriptor = { entries: maxResourceBindings };

    // Control
    t.device.createBindGroupLayout(goodDescriptor);

    const newDescriptor = clone(goodDescriptor);
    newDescriptor.entries.push({
      binding: maxedCount,
      visibility: extraVisibility,
      type: extraType,
      storageTextureFormat: extraTypeInfo.resource === 'storageTex' ? 'rgba8unorm' : undefined,
    });

    const shouldError = maxedCount >= kMaxBindingsPerBindGroup;

    t.expectValidationError(() => {
      t.device.createBindGroupLayout(newDescriptor);
    }, shouldError);
  });

// One pipeline layout can have a maximum number of each type of binding *per stage* (which is
// different for each type). Test that the max works, then add one more binding of same-or-different
// type and same-or-different visibility.
g.test('max_resources_per_stage,in_pipeline_layout')
  .params(kCasesForMaxResourcesPerStageTests)
  .fn(async t => {
    const { maxedType, extraType, maxedVisibility, extraVisibility } = t.params;
    const maxedTypeInfo = kBindingTypeInfo[maxedType];
    const maxedCount = maxedTypeInfo.perStageLimitClass.max;
    const extraTypeInfo = kBindingTypeInfo[extraType];

    const maxResourceBindings = [];
    for (let i = 0; i < maxedCount; i++) {
      maxResourceBindings.push({
        binding: i,
        visibility: maxedVisibility,
        type: maxedType,
        storageTextureFormat: maxedTypeInfo.resource === 'storageTex' ? 'rgba8unorm' : undefined,
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
          type: extraType,
          storageTextureFormat: extraTypeInfo.resource === 'storageTex' ? 'rgba8unorm' : undefined,
        },
      ],
    });

    // Some binding types use the same limit, e.g. 'storage-buffer' and 'readonly-storage-buffer'.
    const newBindingCountsTowardSamePerStageLimit =
      (maxedVisibility & extraVisibility) !== 0 &&
      kBindingTypeInfo[maxedType].perStageLimitClass.class ===
        kBindingTypeInfo[extraType].perStageLimitClass.class;
    const layoutExceedsPerStageLimit = newBindingCountsTowardSamePerStageLimit;

    t.expectValidationError(() => {
      t.device.createPipelineLayout({ bindGroupLayouts: [goodLayout, extraLayout] });
    }, layoutExceedsPerStageLimit);
  });
