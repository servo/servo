/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createBindGroup validation tests.
`;
import * as C from '../../../common/constants.js';
import { pcombine, poptions } from '../../../common/framework/params.js';
import { TestGroup } from '../../../common/framework/test_group.js';
import { unreachable } from '../../../common/framework/util/util.js';
import { kBindingTypes } from '../../capability_info.js';
import { BindingResourceType, ValidationTest, resourceBindingMatches } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = new TestGroup(ValidationTest);
g.test('binding count mismatch', async t => {
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  });
  const goodDescriptor = {
    entries: [{
      binding: 0,
      resource: {
        buffer: t.getStorageBuffer()
      }
    }],
    layout: bindGroupLayout
  }; // Control case

  t.device.createBindGroup(goodDescriptor); // Another binding is not expected.

  const badDescriptor = {
    entries: [{
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
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }]
  });
  const goodDescriptor = {
    entries: [{
      binding: 0,
      resource: {
        buffer: t.getStorageBuffer()
      }
    }],
    layout: bindGroupLayout
  }; // Control case

  t.device.createBindGroup(goodDescriptor); // Binding index 0 must be present.

  const badDescriptor = {
    entries: [{
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
g.test('buffer binding must contain exactly one buffer of its type', t => {
  const bindingType = t.params.bindingType;
  const resourceType = t.params.resourceType;
  const layout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: bindingType
    }]
  });
  const resource = t.getBindingResource(resourceType);
  const shouldError = !resourceBindingMatches(bindingType, resourceType);
  t.expectValidationError(() => {
    t.device.createBindGroup({
      layout,
      entries: [{
        binding: 0,
        resource
      }]
    });
  }, shouldError);
}).params(pcombine(poptions('bindingType', kBindingTypes), poptions('resourceType', Object.keys(BindingResourceType))));
g.test('texture binding must have correct usage', async t => {
  const type = t.params.type;
  const usage = t.params._usage;
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type
    }]
  });
  const goodDescriptor = {
    size: {
      width: 16,
      height: 16,
      depth: 1
    },
    format: C.TextureFormat.R8Unorm,
    usage
  }; // Control case

  t.device.createBindGroup({
    entries: [{
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

    if (type !== 'readonly-storage-texture' && type !== 'writeonly-storage-texture') {
      yield GPUTextureUsage.STORAGE;
    }

    yield GPUTextureUsage.OUTPUT_ATTACHMENT;
  } // Mismatched texture binding usages are not valid.


  for (const mismatchedTextureUsage of mismatchedTextureUsages()) {
    const badDescriptor = clone(goodDescriptor);
    badDescriptor.usage = mismatchedTextureUsage;
    t.expectValidationError(() => {
      t.device.createBindGroup({
        entries: [{
          binding: 0,
          resource: t.device.createTexture(badDescriptor).createView()
        }],
        layout: bindGroupLayout
      });
    });
  }
}).params([{
  type: 'sampled-texture',
  _usage: C.TextureUsage.Sampled
}, {
  type: 'storage-texture',
  _usage: C.TextureUsage.Storage
}]);
g.test('texture must have correct component type', async t => {
  const {
    textureComponentType
  } = t.params;
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{
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
    unreachable('Unexpected texture component type');
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
    entries: [{
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
        entries: [{
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
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type: 'sampled-texture',
      viewDimension: '2d'
    }]
  });
  const goodDescriptor = {
    size: {
      width: 16,
      height: 16,
      depth: 1
    },
    format: C.TextureFormat.RGBA8Unorm,
    usage: GPUTextureUsage.SAMPLED
  }; // Control case

  t.device.createBindGroup({
    entries: [{
      binding: 0,
      resource: t.device.createTexture(goodDescriptor).createView()
    }],
    layout: bindGroupLayout
  }); // Mismatched texture binding formats are not valid.

  const badDescriptor = clone(goodDescriptor);
  badDescriptor.size.depth = 2;
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{
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
    entries: [{
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
    entries: [{
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
// Touching the end of the buffer
{
  offset: 0,
  size: 1024,
  _success: true
}, {
  offset: 0,
  size: undefined,
  _success: true
}, {
  offset: 256 * 3,
  size: 256,
  _success: true
}, {
  offset: 256 * 3,
  size: undefined,
  _success: true
}, // Zero-sized bindings
{
  offset: 0,
  size: 0,
  _success: true
}, {
  offset: 256,
  size: 0,
  _success: true
}, {
  offset: 1024,
  size: 0,
  _success: true
}, {
  offset: 1024,
  size: undefined,
  _success: true
}, // Unaligned buffer offset is invalid
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
}, // Out-of-bounds
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
} // offset+size is OOB
]);
//# sourceMappingURL=createBindGroup.spec.js.map