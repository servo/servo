/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createBindGroup validation tests.
`;
import { TestGroup, pcombine, poptions } from '../../../framework/index.js';
import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

class F extends ValidationTest {
  getStorageBuffer() {
    return this.device.createBuffer({
      size: 1024,
      usage: GPUBufferUsage.STORAGE
    });
  }

  getUniformBuffer() {
    return this.device.createBuffer({
      size: 1024,
      usage: GPUBufferUsage.UNIFORM
    });
  }

  getSampler() {
    return this.device.createSampler();
  }

  getSampledTexture() {
    return this.device.createTexture({
      size: {
        width: 16,
        height: 16,
        depth: 1
      },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.SAMPLED
    });
  }

  getStorageTexture() {
    return this.device.createTexture({
      size: {
        width: 16,
        height: 16,
        depth: 1
      },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.STORAGE
    });
  }

}

export const g = new TestGroup(F);
g.test('binding count mismatch', async t => {
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  });
  const goodDescriptor = {
    bindings: [{
      binding: 0,
      resource: {
        buffer: t.getStorageBuffer()
      }
    }],
    layout: bindGroupLayout
  }; // Control case

  t.device.createBindGroup(goodDescriptor); // Another binding is not expected.

  const badDescriptor = {
    bindings: [{
      binding: 0,
      resource: {
        buffer: t.getStorageBuffer()
      }
    }, // Another binding is added.
    {
      binding: 1,
      resource: {
        buffer: t.getStorageBuffer()
      }
    }],
    layout: bindGroupLayout
  };
  t.expectValidationError(() => {
    t.device.createBindGroup(badDescriptor);
  });
});
g.test('binding must be present in layout', async t => {
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  });
  const goodDescriptor = {
    bindings: [{
      binding: 0,
      resource: {
        buffer: t.getStorageBuffer()
      }
    }],
    layout: bindGroupLayout
  }; // Control case

  t.device.createBindGroup(goodDescriptor); // Binding index 0 must be present.

  const badDescriptor = {
    bindings: [{
      binding: 1,
      // binding index becomes 1.
      resource: {
        buffer: t.getStorageBuffer()
      }
    }],
    layout: bindGroupLayout
  };
  t.expectValidationError(() => {
    t.device.createBindGroup(badDescriptor);
  });
});
g.test('buffer binding must contain exactly one buffer of its type', async t => {
  const {
    bindingType,
    resourceType
  } = t.params;
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: bindingType
    }]
  });
  let resource;

  if (resourceType === 'error') {
    resource = {
      buffer: await t.getErrorBuffer()
    };
  } else if (resourceType === 'uniform-buffer') {
    resource = {
      buffer: t.getUniformBuffer()
    };
  } else if (resourceType === 'storage-buffer') {
    resource = {
      buffer: t.getStorageBuffer()
    };
  } else if (resourceType === 'sampler') {
    resource = t.getSampler();
  } else if (resourceType === 'sampled-texture') {
    resource = t.getSampledTexture().createView();
  } else if (resourceType === 'storage-texture') {
    resource = t.getStorageTexture().createView();
  } else throw new Error();

  let shouldError = bindingType !== resourceType;

  if (bindingType === 'readonly-storage-buffer' && resourceType === 'storage-buffer') {
    shouldError = false;
  }

  t.expectValidationError(() => {
    t.device.createBindGroup({
      bindings: [{
        binding: 0,
        resource
      }],
      layout: bindGroupLayout
    });
  }, shouldError);
}).params(pcombine(poptions('bindingType', ['uniform-buffer', 'storage-buffer', 'readonly-storage-buffer', 'sampler', 'sampled-texture', 'storage-texture']), poptions('resourceType', ['error', 'uniform-buffer', 'storage-buffer', 'sampler', 'sampled-texture', 'storage-texture'])));
g.test('texture binding must have correct usage', async t => {
  const {
    type
  } = t.params;
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type
    }]
  });
  let usage;

  if (type === 'sampled-texture') {
    usage = GPUTextureUsage.SAMPLED;
  } else if (type === 'storage-texture') {
    usage = GPUTextureUsage.STORAGE;
  } else {
    throw new Error('Unexpected binding type');
  }

  const goodDescriptor = {
    size: {
      width: 16,
      height: 16,
      depth: 1
    },
    format: 'r8unorm',
    usage
  }; // Control case

  t.device.createBindGroup({
    bindings: [{
      binding: 0,
      resource: t.device.createTexture(goodDescriptor).createView()
    }],
    layout: bindGroupLayout
  });

  function* mismatchedTextureUsages() {
    yield GPUTextureUsage.COPY_SRC;
    yield GPUTextureUsage.COPY_DST;

    if (type !== 'sampled-texture') {
      yield GPUTextureUsage.SAMPLED;
    }

    if (type !== 'storage-texture') {
      yield GPUTextureUsage.STORAGE;
    }

    yield GPUTextureUsage.OUTPUT_ATTACHMENT;
  } // Mismatched texture binding usages are not valid.


  for (const mismatchedTextureUsage of mismatchedTextureUsages()) {
    const badDescriptor = clone(goodDescriptor);
    badDescriptor.usage = mismatchedTextureUsage;
    t.expectValidationError(() => {
      t.device.createBindGroup({
        bindings: [{
          binding: 0,
          resource: t.device.createTexture(badDescriptor).createView()
        }],
        layout: bindGroupLayout
      });
    });
  }
}).params(poptions('type', ['sampled-texture', 'storage-texture']));
g.test('texture must have correct component type', async t => {
  const {
    textureComponentType
  } = t.params;
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type: 'sampled-texture',
      textureComponentType
    }]
  }); // TODO: Test more texture component types.

  let format;

  if (textureComponentType === 'float') {
    format = 'r8unorm';
  } else if (textureComponentType === 'sint') {
    format = 'r8sint';
  } else if (textureComponentType === 'uint') {
    format = 'r8uint';
  } else {
    throw new Error('Unexpected texture component type');
  }

  const goodDescriptor = {
    size: {
      width: 16,
      height: 16,
      depth: 1
    },
    format,
    usage: GPUTextureUsage.SAMPLED
  }; // Control case

  t.device.createBindGroup({
    bindings: [{
      binding: 0,
      resource: t.device.createTexture(goodDescriptor).createView()
    }],
    layout: bindGroupLayout
  });

  function* mismatchedTextureFormats() {
    if (textureComponentType !== 'float') {
      yield 'r8unorm';
    }

    if (textureComponentType !== 'sint') {
      yield 'r8sint';
    }

    if (textureComponentType !== 'uint') {
      yield 'r8uint';
    }
  } // Mismatched texture binding formats are not valid.


  for (const mismatchedTextureFormat of mismatchedTextureFormats()) {
    const badDescriptor = clone(goodDescriptor);
    badDescriptor.format = mismatchedTextureFormat;
    t.expectValidationError(() => {
      t.device.createBindGroup({
        bindings: [{
          binding: 0,
          resource: t.device.createTexture(badDescriptor).createView()
        }],
        layout: bindGroupLayout
      });
    });
  }
}).params(poptions('textureComponentType', ['float', 'sint', 'uint'])); // TODO: Write test for all dimensions.

g.test('texture must have correct dimension', async t => {
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type: 'sampled-texture',
      textureDimension: '2d'
    }]
  });
  const goodDescriptor = {
    size: {
      width: 16,
      height: 16,
      depth: 1
    },
    arrayLayerCount: 1,
    format: 'rgba8unorm',
    usage: GPUTextureUsage.SAMPLED
  }; // Control case

  t.device.createBindGroup({
    bindings: [{
      binding: 0,
      resource: t.device.createTexture(goodDescriptor).createView()
    }],
    layout: bindGroupLayout
  }); // Mismatched texture binding formats are not valid.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.arrayLayerCount = 2;
  t.expectValidationError(() => {
    t.device.createBindGroup({
      bindings: [{
        binding: 0,
        resource: t.device.createTexture(badDescriptor).createView()
      }],
      layout: bindGroupLayout
    });
  });
});
g.test('buffer offset and size for bind groups match', async t => {
  const {
    offset,
    size,
    _success
  } = t.params;
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  });
  const buffer = t.device.createBuffer({
    size: 1024,
    usage: GPUBufferUsage.STORAGE
  });
  const descriptor = {
    bindings: [{
      binding: 0,
      resource: {
        buffer,
        offset,
        size
      }
    }],
    layout: bindGroupLayout
  };

  if (_success) {
    // Control case
    t.device.createBindGroup(descriptor);
  } else {
    // Buffer offset and/or size don't match in bind groups.
    t.expectValidationError(() => {
      t.device.createBindGroup(descriptor);
    });
  }
}).params([{
  offset: 0,
  size: 512,
  _success: true
}, // offset 0 is valid
{
  offset: 256,
  size: 256,
  _success: true
}, // offset 256 (aligned) is valid
// unaligned buffer offset is invalid
{
  offset: 1,
  size: 256,
  _success: false
}, {
  offset: 1,
  size: undefined,
  _success: false
}, {
  offset: 128,
  size: 256,
  _success: false
}, {
  offset: 255,
  size: 256,
  _success: false
}, {
  offset: 0,
  size: 256,
  _success: true
}, // touching the start of the buffer works
{
  offset: 256 * 3,
  size: 256,
  _success: true
}, // touching the end of the buffer works
{
  offset: 1024,
  size: 0,
  _success: true
}, // touching the end of the buffer works
{
  offset: 0,
  size: 1024,
  _success: true
}, // touching the full buffer works
{
  offset: 0,
  size: undefined,
  _success: true
}, // touching the full buffer works
{
  offset: 256 * 5,
  size: 0,
  _success: false
}, // offset is OOB
{
  offset: 0,
  size: 256 * 5,
  _success: false
}, // size is OOB
{
  offset: 1024,
  size: 1,
  _success: false
}, // offset+size is OOB
{
  offset: 256,
  size: -256,
  _success: false
} // offset+size overflows to be 0
]);
//# sourceMappingURL=createBindGroup.spec.js.map