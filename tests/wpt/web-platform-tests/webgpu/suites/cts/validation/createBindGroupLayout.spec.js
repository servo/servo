/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createBindGroupLayout validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
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
      type: 'storage-buffer'
    }, {
      binding: 1,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  }; // Control case

  t.device.createBindGroupLayout(goodDescriptor);
  const badDescriptor = clone(goodDescriptor);
  badDescriptor.bindings[1].binding = 0; // Binding index 0 can't be specified twice.

  await t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
});
g.test('negative binding index', async t => {
  const goodDescriptor = {
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  }; // Control case

  t.device.createBindGroupLayout(goodDescriptor); // Negative binding index can't be specified.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.bindings[0].binding = -1;
  await t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
});
g.test('Visibility of bindings can be 0', async t => {
  const descriptor = {
    bindings: [{
      binding: 0,
      visibility: 0,
      type: 'storage-buffer'
    }]
  };
  t.device.createBindGroupLayout(descriptor);
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
  await t.expectValidationError(() => {
    t.device.createBindGroupLayout(badDescriptor);
  });
}).params([{
  type: 'storage-buffer',
  maxDynamicBufferCount: 4
}, {
  type: 'uniform-buffer',
  maxDynamicBufferCount: 8
}]);
g.test('dynamic set to true is allowed only for buffers', async t => {
  const {
    type,
    success
  } = t.params;
  const descriptor = {
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type,
      hasDynamicOffset: true
    }]
  };

  if (success) {
    // Control case
    t.device.createBindGroupLayout(descriptor);
  } else {
    // Dynamic set to true is not allowed in some cases.
    await t.expectValidationError(() => {
      t.device.createBindGroupLayout(descriptor);
    });
  }
}).params([{
  type: 'uniform-buffer',
  success: true
}, {
  type: 'storage-buffer',
  success: true
}, {
  type: 'readonly-storage-buffer',
  success: true
}, {
  type: 'sampler',
  success: false
}, {
  type: 'sampled-texture',
  success: false
}, {
  type: 'storage-texture',
  success: false
}]);
//# sourceMappingURL=createBindGroupLayout.spec.js.map