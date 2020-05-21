/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
createBindGroup validation tests.
`;
import * as C from '../../../common/constants.js';
import { poptions, params } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { unreachable } from '../../../common/framework/util/util.js';
import { kBindingTypes, kBindingTypeInfo, kBindableResources, kTextureUsages, kTextureBindingTypes, kTextureBindingTypeInfo } from '../../capability_info.js';
import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = makeTestGroup(ValidationTest);
g.test('binding_count_mismatch').fn(async t => {
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
g.test('binding_must_be_present_in_layout').fn(async t => {
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
g.test('buffer_binding_must_contain_exactly_one_buffer_of_its_type').params(params().combine(poptions('bindingType', kBindingTypes)).combine(poptions('resourceType', kBindableResources))).fn(t => {
  const {
    bindingType,
    resourceType
  } = t.params;
  const info = kBindingTypeInfo[bindingType];
  const storageTextureFormat = info.resource === 'storageTex' ? 'rgba8unorm' : undefined;
  const layout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: bindingType,
      storageTextureFormat
    }]
  });
  const resource = t.getBindingResource(resourceType);
  const resourceBindingMatches = info.resource === resourceType;
  t.expectValidationError(() => {
    t.device.createBindGroup({
      layout,
      entries: [{
        binding: 0,
        resource
      }]
    });
  }, !resourceBindingMatches);
});
g.test('texture_binding_must_have_correct_usage').params(params().combine(poptions('type', kTextureBindingTypes)).combine(poptions('usage', kTextureUsages))).fn(async t => {
  const {
    type,
    usage
  } = t.params;
  const info = kTextureBindingTypeInfo[type];
  const storageTextureFormat = info.resource === 'storageTex' ? 'rgba8unorm' : undefined;
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type,
      storageTextureFormat
    }]
  });
  const descriptor = {
    size: {
      width: 16,
      height: 16,
      depth: 1
    },
    format: C.TextureFormat.RGBA8Unorm,
    usage
  };
  const shouldError = usage !== info.usage;
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{
        binding: 0,
        resource: t.device.createTexture(descriptor).createView()
      }],
      layout: bindGroupLayout
    });
  }, shouldError);
});
g.test('texture_must_have_correct_component_type').params(poptions('textureComponentType', [C.TextureComponentType.Float, C.TextureComponentType.Sint, C.TextureComponentType.Uint])).fn(async t => {
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
}); // TODO: Write test for all dimensions.

g.test('texture_must_have_correct_dimension').fn(async t => {
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
g.test('buffer_offset_and_size_for_bind_groups_match').params([{
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
]).fn(async t => {
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
});
//# sourceMappingURL=createBindGroup.spec.js.map