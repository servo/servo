/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createBindGroupLayout validation tests.
`;
import { C, TestGroup, poptions } from '../../../framework/index.js';
import { bindingTypeInfo, bindingTypes } from '../format_info.js';
import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = new TestGroup(ValidationTest);
g.test('some binding index was specified more than once', async t => {
  const goodDescriptor = {
    bindings: [{
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
  badDescriptor.bindings[1].binding = 0; // Binding index 0 can't be specified twice.

  t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
});
g.test('negative binding index', async t => {
  const goodDescriptor = {
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: C.BindingType.StorageBuffer
    }]
  }; // Control case

  t.device.createBindGroupLayout(goodDescriptor); // Negative binding index can't be specified.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.bindings[0].binding = -1;
  t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
});
g.test('Visibility of bindings can be 0', async t => {
  t.device.createBindGroupLayout({
    bindings: [{
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
    bindings: [...maxDynamicBufferBindings, {
      binding: maxDynamicBufferBindings.length,
      visibility: GPUShaderStage.COMPUTE,
      type,
      hasDynamicOffset: false
    }]
  }; // Control case

  t.device.createBindGroupLayout(goodDescriptor); // Dynamic buffers exceed maximum in a bind group layout.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.bindings[maxDynamicBufferCount].hasDynamicOffset = true;
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
  const success = bindingTypeInfo[type].type === 'buffer';
  const descriptor = {
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type,
      hasDynamicOffset: true
    }]
  };
  t.expectValidationError(() => {
    t.device.createBindGroupLayout(descriptor);
  }, !success);
}).params(poptions('type', bindingTypes));
//# sourceMappingURL=createBindGroupLayout.spec.js.map