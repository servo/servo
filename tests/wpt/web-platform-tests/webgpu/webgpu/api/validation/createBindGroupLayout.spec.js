/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createBindGroupLayout validation tests.
`;
import * as C from '../../../common/constants.js';
import { poptions } from '../../../common/framework/params.js';
import { TestGroup } from '../../../common/framework/test_group.js';
import { kBindingTypeInfo, kBindingTypes, kMaxBindingsPerBindGroup, kPerStageBindingLimits, kShaderStages } from '../../capability_info.js';
import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = new TestGroup(ValidationTest);
g.test('some binding index was specified more than once', async t => {
  const goodDescriptor = {
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: C.BindingType.StorageBuffer
    }, {
      binding: 1,
      visibility: GPUShaderStage.COMPUTE,
      type: C.BindingType.StorageBuffer
    }]
  }; // Control case

  t.device.createBindGroupLayout(goodDescriptor);
  const badDescriptor = clone(goodDescriptor);
  badDescriptor.entries[1].binding = 0; // Binding index 0 can't be specified twice.

  t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
});
g.test('Visibility of bindings can be 0', async t => {
  t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: 0,
      type: 'storage-buffer'
    }]
  });
});
g.test('number of dynamic buffers exceeds the maximum value', async t => {
  const {
    type,
    maxDynamicBufferCount
  } = t.params;
  const maxDynamicBufferBindings = [];

  for (let i = 0; i < maxDynamicBufferCount; i++) {
    maxDynamicBufferBindings.push({
      binding: i,
      visibility: GPUShaderStage.COMPUTE,
      type,
      hasDynamicOffset: true
    });
  }

  const goodDescriptor = {
    entries: [...maxDynamicBufferBindings, {
      binding: maxDynamicBufferBindings.length,
      visibility: GPUShaderStage.COMPUTE,
      type,
      hasDynamicOffset: false
    }]
  }; // Control case

  t.device.createBindGroupLayout(goodDescriptor); // Dynamic buffers exceed maximum in a bind group layout.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.entries[maxDynamicBufferCount].hasDynamicOffset = true;
  t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
}).params([{
  type: C.BindingType.StorageBuffer,
  maxDynamicBufferCount: 4
}, {
  type: C.BindingType.UniformBuffer,
  maxDynamicBufferCount: 8
}]);
g.test('dynamic set to true is allowed only for buffers', async t => {
  const type = t.params.type;
  const success = kBindingTypeInfo[type].type === 'buffer';
  const descriptor = {
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type,
      hasDynamicOffset: true
    }]
  };
  t.expectValidationError(() => {
    t.device.createBindGroupLayout(descriptor);
  }, !success);
}).params(poptions('type', kBindingTypes));
let kCasesForMaxResourcesPerStageTests;
{
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
        if (info.perStageLimitType === kBindingTypeInfo[extraBindingType].perStageLimitType) {
          yield extraBindingType;
        }
      }
    } else {
      yield info.perStageLimitType === 'sampler' ? 'sampled-texture' : 'sampler';
    }
  }

  kCasesForMaxResourcesPerStageTests = [];

  for (const maxedType of kBindingTypes) {
    for (const maxedVisibility of kShaderStages) {
      // Don't generate a case where maxedType isn't valid in maxedVisibility.
      if (!(kBindingTypeInfo[maxedType].validStages & maxedVisibility)) continue;

      for (const extraTypeSame of [true, false]) {
        for (const extraType of pickExtraBindingTypes(maxedType, extraTypeSame)) {
          for (const extraVisibility of kShaderStages) {
            // Don't generate a case where extraType isn't valid in extraVisibility.
            if (!(kBindingTypeInfo[extraType].validStages & extraVisibility)) continue;
            kCasesForMaxResourcesPerStageTests.push({
              maxedType,
              maxedVisibility,
              extraType,
              extraVisibility
            });
          }
        }
      }
    }
  }
} // Should never fail unless kMaxBindingsPerBindGroup is exceeded, because the validation for
// resources-of-type-per-stage is in pipeline layout creation.

g.test('max resources per stage/in bind group layout', async t => {
  const maxedType = t.params.maxedType;
  const extraType = t.params.extraType;
  const {
    maxedVisibility,
    extraVisibility
  } = t.params;
  const maxedCount = kPerStageBindingLimits[kBindingTypeInfo[maxedType].perStageLimitType];
  const maxResourceBindings = [];

  for (let i = 0; i < maxedCount; i++) {
    maxResourceBindings.push({
      binding: i,
      visibility: maxedVisibility,
      type: maxedType
    });
  }

  const goodDescriptor = {
    entries: maxResourceBindings
  }; // Control

  t.device.createBindGroupLayout(goodDescriptor);
  const newDescriptor = clone(goodDescriptor);
  newDescriptor.entries.push({
    binding: maxedCount,
    visibility: extraVisibility,
    type: extraType
  });
  const shouldError = maxedCount >= kMaxBindingsPerBindGroup;
  t.expectValidationError(() => {
    t.device.createBindGroupLayout(newDescriptor);
  }, shouldError);
}).params(kCasesForMaxResourcesPerStageTests); // One pipeline layout can have a maximum number of each type of binding *per stage* (which is
// different for each type). Test that the max works, then add one more binding of same-or-different
// type and same-or-different visibility.

g.test('max resources per stage/in pipeline layout', async t => {
  const maxedType = t.params.maxedType;
  const extraType = t.params.extraType;
  const {
    maxedVisibility,
    extraVisibility
  } = t.params;
  const maxedCount = kPerStageBindingLimits[kBindingTypeInfo[maxedType].perStageLimitType];
  const maxResourceBindings = [];

  for (let i = 0; i < maxedCount; i++) {
    maxResourceBindings.push({
      binding: i,
      visibility: maxedVisibility,
      type: maxedType
    });
  }

  const goodLayout = t.device.createBindGroupLayout({
    entries: maxResourceBindings
  }); // Control

  t.device.createPipelineLayout({
    bindGroupLayouts: [goodLayout]
  });
  const extraLayout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: extraVisibility,
      type: extraType
    }]
  }); // Some binding types use the same limit, e.g. 'storage-buffer' and 'readonly-storage-buffer'.

  const newBindingCountsTowardSamePerStageLimit = (maxedVisibility & extraVisibility) !== 0 && kBindingTypeInfo[maxedType].perStageLimitType === kBindingTypeInfo[extraType].perStageLimitType;
  const layoutExceedsPerStageLimit = newBindingCountsTowardSamePerStageLimit;
  t.expectValidationError(() => {
    t.device.createPipelineLayout({
      bindGroupLayouts: [goodLayout, extraLayout]
    });
  }, layoutExceedsPerStageLimit);
}).params(kCasesForMaxResourcesPerStageTests);
//# sourceMappingURL=createBindGroupLayout.spec.js.map