/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createPipelineLayout validation tests.
`;
import { pbool, pcombine, poptions } from '../../../common/framework/params.js';
import { TestGroup } from '../../../common/framework/test_group.js';
import { kBindingTypeInfo, kBindingTypes, kShaderStageCombinations } from '../../capability_info.js';
import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = new TestGroup(ValidationTest);
g.test('number of dynamic buffers exceeds the maximum value', async t => {
  const {
    type,
    visibility
  } = t.params;
  const maxDynamicCount = kBindingTypeInfo[type].maxDynamicCount;
  const maxDynamicBufferBindings = [];

  for (let binding = 0; binding < maxDynamicCount; binding++) {
    maxDynamicBufferBindings.push({
      binding,
      visibility,
      type,
      hasDynamicOffset: true
    });
  }

  const maxDynamicBufferBindGroupLayout = t.device.createBindGroupLayout({
    entries: maxDynamicBufferBindings
  });
  const goodDescriptor = {
    entries: [{
      binding: 0,
      visibility,
      type,
      hasDynamicOffset: false
    }]
  };
  const goodPipelineLayoutDescriptor = {
    bindGroupLayouts: [maxDynamicBufferBindGroupLayout, t.device.createBindGroupLayout(goodDescriptor)]
  }; // Control case

  t.device.createPipelineLayout(goodPipelineLayoutDescriptor); // Check dynamic buffers exceed maximum in pipeline layout.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.entries[0].hasDynamicOffset = true;
  const badPipelineLayoutDescriptor = {
    bindGroupLayouts: [maxDynamicBufferBindGroupLayout, t.device.createBindGroupLayout(badDescriptor)]
  };
  t.expectValidationError(() => {
    t.device.createPipelineLayout(badPipelineLayoutDescriptor);
  });
}).params(pcombine(poptions('visibility', [0, 2, 4, 6]), //
poptions('type', ['uniform-buffer', 'storage-buffer', 'readonly-storage-buffer'])));
g.test('visibility and dynamic offsets', t => {
  const hasDynamicOffset = t.params.hasDynamicOffset;
  const type = t.params.type;
  const visibility = t.params.visibility;
  const info = kBindingTypeInfo[type];
  const descriptor = {
    entries: [{
      binding: 0,
      visibility,
      type,
      hasDynamicOffset
    }]
  };
  let success = true;
  if (info.type !== 'buffer' && hasDynamicOffset) success = false;
  if ((visibility & ~info.validStages) !== 0) success = false;
  t.expectValidationError(() => {
    t.device.createPipelineLayout({
      bindGroupLayouts: [t.device.createBindGroupLayout(descriptor)]
    });
  }, !success);
}).params(pcombine(poptions('type', kBindingTypes), //
pbool('hasDynamicOffset'), poptions('visibility', kShaderStageCombinations)));
g.test('number of bind group layouts exceeds the maximum value', async t => {
  const bindGroupLayoutDescriptor = {
    entries: []
  }; // 4 is the maximum number of bind group layouts.

  const maxBindGroupLayouts = [1, 2, 3, 4].map(() => t.device.createBindGroupLayout(bindGroupLayoutDescriptor));
  const goodPipelineLayoutDescriptor = {
    bindGroupLayouts: maxBindGroupLayouts
  }; // Control case

  t.device.createPipelineLayout(goodPipelineLayoutDescriptor); // Check bind group layouts exceed maximum in pipeline layout.

  const badPipelineLayoutDescriptor = {
    bindGroupLayouts: [...maxBindGroupLayouts, t.device.createBindGroupLayout(bindGroupLayoutDescriptor)]
  };
  t.expectValidationError(() => {
    t.device.createPipelineLayout(badPipelineLayoutDescriptor);
  });
});
//# sourceMappingURL=createPipelineLayout.spec.js.map