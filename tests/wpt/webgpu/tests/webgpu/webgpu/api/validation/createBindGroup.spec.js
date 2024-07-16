/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
  createBindGroup validation tests.

  TODO: Ensure sure tests cover all createBindGroup validation rules.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert, makeValueTestVariant, unreachable } from '../../../common/util/util.js';
import {
  allBindingEntries,

  bindingTypeInfo,
  bufferBindingEntries,
  bufferBindingTypeInfo,
  kBindableResources,
  kBufferBindingTypes,
  kBufferUsages,
  kCompareFunctions,
  kSamplerBindingTypes,
  kTextureUsages,
  kTextureViewDimensions,
  sampledAndStorageBindingEntries,
  texBindingTypeInfo } from
'../../capability_info.js';
import { GPUConst } from '../../constants.js';
import { kAllTextureFormats, kTextureFormatInfo } from '../../format_info.js';
import { kResourceStates } from '../../gpu_test.js';
import { getTextureDimensionFromView } from '../../util/texture/base.js';

import { ValidationTest } from './validation_test.js';

function clone(descriptor) {
  return JSON.parse(JSON.stringify(descriptor));
}

export const g = makeTestGroup(ValidationTest);

const kStorageTextureFormats = kAllTextureFormats.filter((f) => kTextureFormatInfo[f].color?.storage);

g.test('binding_count_mismatch').
desc('Test that the number of entries must match the number of entries in the BindGroupLayout.').
paramsSubcasesOnly((u) =>
u //
.combine('layoutEntryCount', [1, 2, 3]).
combine('bindGroupEntryCount', [1, 2, 3])
).
fn((t) => {
  const { layoutEntryCount, bindGroupEntryCount } = t.params;

  const layoutEntries = [];
  for (let i = 0; i < layoutEntryCount; ++i) {
    layoutEntries.push({
      binding: i,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type: 'storage' }
    });
  }
  const bindGroupLayout = t.device.createBindGroupLayout({ entries: layoutEntries });

  const entries = [];
  for (let i = 0; i < bindGroupEntryCount; ++i) {
    entries.push({
      binding: i,
      resource: { buffer: t.getStorageBuffer() }
    });
  }

  const shouldError = layoutEntryCount !== bindGroupEntryCount;
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries,
      layout: bindGroupLayout
    });
  }, shouldError);
});

g.test('binding_must_be_present_in_layout').
desc(
  'Test that the binding slot for each entry matches a binding slot defined in the BindGroupLayout.'
).
paramsSubcasesOnly((u) =>
u //
.combine('layoutBinding', [0, 1, 2]).
combine('binding', [0, 1, 2])
).
fn((t) => {
  const { layoutBinding, binding } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    { binding: layoutBinding, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } }]

  });

  const descriptor = {
    entries: [{ binding, resource: { buffer: t.getStorageBuffer() } }],
    layout: bindGroupLayout
  };

  const shouldError = layoutBinding !== binding;
  t.expectValidationError(() => {
    t.device.createBindGroup(descriptor);
  }, shouldError);
});

g.test('binding_must_contain_resource_defined_in_layout').
desc(
  'Test that only compatible resource types specified in the BindGroupLayout are allowed for each entry.'
).
params((u) =>
u //
.combine('resourceType', kBindableResources).
combine('entry', allBindingEntries(false))
).
fn((t) => {
  const { resourceType, entry } = t.params;
  const info = bindingTypeInfo(entry);

  const layout = t.device.createBindGroupLayout({
    entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, ...entry }]
  });

  const resource = t.getBindingResource(resourceType);

  const IsStorageTextureResourceType = (resourceType) => {
    switch (resourceType) {
      case 'readonlyStorageTex':
      case 'readwriteStorageTex':
      case 'writeonlyStorageTex':
        return true;
      default:
        return false;
    }
  };

  let resourceBindingIsCompatible;
  switch (info.resource) {
    // Either type of sampler may be bound to a filtering sampler binding.
    case 'filtSamp':
      resourceBindingIsCompatible = resourceType === 'filtSamp' || resourceType === 'nonFiltSamp';
      break;
    // But only non-filtering samplers can be used with non-filtering sampler bindings.
    case 'nonFiltSamp':
      resourceBindingIsCompatible = resourceType === 'nonFiltSamp';
      break;
    case 'readonlyStorageTex':
    case 'readwriteStorageTex':
    case 'writeonlyStorageTex':
      resourceBindingIsCompatible = IsStorageTextureResourceType(resourceType);
      break;
    default:
      resourceBindingIsCompatible = info.resource === resourceType;
      break;
  }
  t.expectValidationError(() => {
    t.device.createBindGroup({ layout, entries: [{ binding: 0, resource }] });
  }, !resourceBindingIsCompatible);
});

g.test('texture_binding_must_have_correct_usage').
desc('Tests that texture bindings must have the correct usage.').
paramsSubcasesOnly((u) =>
u //
.combine('entry', sampledAndStorageBindingEntries(false)).
combine('usage', kTextureUsages).
unless(({ entry, usage }) => {
  const info = texBindingTypeInfo(entry);
  // Can't create the texture for this (usage=STORAGE_BINDING and sampleCount=4), so skip.
  return usage === GPUConst.TextureUsage.STORAGE_BINDING && info.resource === 'sampledTexMS';
})
).
fn((t) => {
  const { entry, usage } = t.params;
  const info = texBindingTypeInfo(entry);

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{ binding: 0, visibility: GPUShaderStage.FRAGMENT, ...entry }]
  });

  // The `RENDER_ATTACHMENT` usage must be specified if sampleCount > 1 according to WebGPU SPEC.
  const appliedUsage =
  info.resource === 'sampledTexMS' ? usage | GPUConst.TextureUsage.RENDER_ATTACHMENT : usage;

  const descriptor = {
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: 'r32float',
    usage: appliedUsage,
    sampleCount: info.resource === 'sampledTexMS' ? 4 : 1
  };
  const resource = t.createTextureTracked(descriptor).createView();

  const shouldError = (usage & info.usage) === 0;
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource }],
      layout: bindGroupLayout
    });
  }, shouldError);
});

g.test('texture_must_have_correct_component_type').
desc(
  `
    Tests that texture bindings must have a format that matches the sample type specified in the BindGroupLayout.
    - Tests a compatible format for every sample type
    - Tests an incompatible format for every sample type`
).
params((u) => u.combine('sampleType', ['float', 'sint', 'uint'])).
fn((t) => {
  const { sampleType } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      texture: { sampleType }
    }]

  });

  let format;
  if (sampleType === 'float') {
    format = 'r8unorm';
  } else if (sampleType === 'sint') {
    format = 'r8sint';
  } else if (sampleType === 'uint') {
    format = 'r8uint';
  } else {
    unreachable('Unexpected texture component type');
  }

  const goodDescriptor = {
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format,
    usage: GPUTextureUsage.TEXTURE_BINDING
  };

  // Control case
  t.device.createBindGroup({
    entries: [
    {
      binding: 0,
      resource: t.createTextureTracked(goodDescriptor).createView()
    }],

    layout: bindGroupLayout
  });

  function* mismatchedTextureFormats() {
    if (sampleType !== 'float') {
      yield 'r8unorm';
    }
    if (sampleType !== 'sint') {
      yield 'r8sint';
    }
    if (sampleType !== 'uint') {
      yield 'r8uint';
    }
  }

  // Mismatched texture binding formats are not valid.
  for (const mismatchedTextureFormat of mismatchedTextureFormats()) {
    const badDescriptor = clone(goodDescriptor);
    badDescriptor.format = mismatchedTextureFormat;

    t.expectValidationError(() => {
      t.device.createBindGroup({
        entries: [{ binding: 0, resource: t.createTextureTracked(badDescriptor).createView() }],
        layout: bindGroupLayout
      });
    });
  }
});

g.test('texture_must_have_correct_dimension').
desc(
  `
    Test that bound texture views match the dimensions supplied in the BindGroupLayout
      - Test for every GPUTextureViewDimension
      - Test for both TEXTURE_BINDING and STORAGE_BINDING.
  `
).
params((u) =>
u.
combine('usage', [
GPUConst.TextureUsage.TEXTURE_BINDING,
GPUConst.TextureUsage.STORAGE_BINDING]
).
combine('viewDimension', kTextureViewDimensions).
unless(
  (p) =>
  p.usage === GPUConst.TextureUsage.STORAGE_BINDING && (
  p.viewDimension === 'cube' || p.viewDimension === 'cube-array')
).
beginSubcases().
combine('dimension', kTextureViewDimensions)
).
fn((t) => {
  const { usage, viewDimension, dimension } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    usage === GPUTextureUsage.TEXTURE_BINDING ?
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      texture: { viewDimension }
    } :
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      storageTexture: { access: 'write-only', format: 'rgba8unorm', viewDimension }
    }]

  });

  let height = 16;
  let depthOrArrayLayers = 6;
  if (dimension === '1d') {
    height = 1;
    depthOrArrayLayers = 1;
  }

  const texture = t.createTextureTracked({
    size: { width: 16, height, depthOrArrayLayers },
    format: 'rgba8unorm',
    usage,
    dimension: getTextureDimensionFromView(dimension)
  });

  t.skipIfTextureViewDimensionNotSupported(viewDimension, dimension);
  if (t.isCompatibility && texture.dimension === '2d') {
    if (depthOrArrayLayers === 1) {
      t.skipIf(
        viewDimension !== '2d',
        '1 layer 2d textures default to textureBindingViewDimension: "2d" in compat mode'
      );
    } else {
      t.skipIf(
        viewDimension !== '2d-array',
        '> 1 layer 2d textures default to textureBindingViewDimension "2d-array" in compat mode'
      );
    }
  }

  const shouldError = viewDimension !== dimension;
  const textureView = texture.createView({ dimension });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: textureView }],
      layout: bindGroupLayout
    });
  }, shouldError);
});

g.test('multisampled_validation').
desc(
  `
    Test that the sample count of the texture is greater than 1 if the BindGroup entry's
    multisampled is true. Otherwise, the texture's sampleCount should be 1.
  `
).
params((u) =>
u //
.combine('multisampled', [true, false]).
beginSubcases().
combine('sampleCount', [1, 4])
).
fn((t) => {
  const { multisampled, sampleCount } = t.params;
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      texture: { multisampled, sampleType: multisampled ? 'unfilterable-float' : undefined }
    }]

  });

  const texture = t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.RENDER_ATTACHMENT,
    sampleCount
  });

  const isValid = !multisampled && sampleCount === 1 || multisampled && sampleCount > 1;

  const textureView = texture.createView();
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: textureView }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('buffer_offset_and_size_for_bind_groups_match').
desc(
  `
    Test that a buffer binding's [offset, offset + size) must be contained in the BindGroup entry's buffer.
    - Test for various offsets and sizes`
).
paramsSubcasesOnly([
{ offset: 0, size: 512, _success: true }, // offset 0 is valid
{ offset: 256, size: 256, _success: true }, // offset 256 (aligned) is valid

// Touching the end of the buffer
{ offset: 0, size: 1024, _success: true },
{ offset: 0, size: undefined, _success: true },
{ offset: 256 * 3, size: 256, _success: true },
{ offset: 256 * 3, size: undefined, _success: true },

// Zero-sized bindings
{ offset: 0, size: 0, _success: false },
{ offset: 256, size: 0, _success: false },
{ offset: 1024, size: 0, _success: false },
{ offset: 1024, size: undefined, _success: false },

// Unaligned buffer offset is invalid
{ offset: 1, size: 256, _success: false },
{ offset: 1, size: undefined, _success: false },
{ offset: 128, size: 256, _success: false },
{ offset: 255, size: 256, _success: false },

// Out-of-bounds
{ offset: 256 * 5, size: 0, _success: false }, // offset is OOB
{ offset: 0, size: 256 * 5, _success: false }, // size is OOB
{ offset: 1024, size: 1, _success: false } // offset+size is OOB
]).
fn((t) => {
  const { offset, size, _success } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [{ binding: 0, visibility: GPUShaderStage.COMPUTE, buffer: { type: 'storage' } }]
  });

  const buffer = t.createBufferTracked({
    size: 1024,
    usage: GPUBufferUsage.STORAGE
  });

  const descriptor = {
    entries: [
    {
      binding: 0,
      resource: { buffer, offset, size }
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

g.test('minBindingSize').
desc('Tests that minBindingSize is correctly enforced.').
paramsSubcasesOnly((u) =>
u //
.combine('minBindingSize', [undefined, 4, 8, 256]).
expand('size', ({ minBindingSize }) =>
minBindingSize !== undefined ?
[minBindingSize - 4, minBindingSize, minBindingSize + 4] :
[4, 256]
)
).
fn((t) => {
  const { size, minBindingSize } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      buffer: {
        type: 'storage',
        minBindingSize
      }
    }]

  });

  const storageBuffer = t.createBufferTracked({
    size,
    usage: GPUBufferUsage.STORAGE
  });

  t.expectValidationError(
    () => {
      t.device.createBindGroup({
        layout: bindGroupLayout,
        entries: [
        {
          binding: 0,
          resource: { buffer: storageBuffer }
        }]

      });
    },
    minBindingSize !== undefined && size < minBindingSize
  );
});

g.test('buffer,resource_state').
desc('Test bind group creation with various buffer resource states').
paramsSubcasesOnly((u) =>
u.combine('state', kResourceStates).combine('entry', bufferBindingEntries(true))
).
fn((t) => {
  const { state, entry } = t.params;

  assert(entry.buffer !== undefined);
  const info = bufferBindingTypeInfo(entry.buffer);

  const bgl = t.device.createBindGroupLayout({
    entries: [
    {
      ...entry,
      binding: 0,
      visibility: info.validStages
    }]

  });

  const buffer = t.createBufferWithState(state, {
    usage: info.usage,
    size: 4
  });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      layout: bgl,
      entries: [
      {
        binding: 0,
        resource: {
          buffer
        }
      }]

    });
  }, state === 'invalid');
});

g.test('texture,resource_state').
desc('Test bind group creation with various texture resource states').
paramsSubcasesOnly((u) =>
u.combine('state', kResourceStates).combine('entry', sampledAndStorageBindingEntries(true))
).
fn((t) => {
  const { state, entry } = t.params;
  const info = texBindingTypeInfo(entry);

  const bgl = t.device.createBindGroupLayout({
    entries: [
    {
      ...entry,
      binding: 0,
      visibility: info.validStages
    }]

  });

  // The `RENDER_ATTACHMENT` usage must be specified if sampleCount > 1 according to WebGPU SPEC.
  const usage = entry.texture?.multisampled ?
  info.usage | GPUConst.TextureUsage.RENDER_ATTACHMENT :
  info.usage;
  const format = entry.storageTexture !== undefined ? 'r32float' : 'rgba8unorm';
  const texture = t.createTextureWithState(state, {
    usage,
    size: [1, 1],
    format,
    sampleCount: entry.texture?.multisampled ? 4 : 1
  });

  let textureView;
  t.expectValidationError(() => {
    textureView = texture.createView();
  }, state === 'invalid');

  t.expectValidationError(() => {
    t.device.createBindGroup({
      layout: bgl,
      entries: [
      {
        binding: 0,
        resource: textureView
      }]

    });
  }, state === 'invalid');
});

g.test('bind_group_layout,device_mismatch').
desc(
  'Tests createBindGroup cannot be called with a bind group layout created from another device'
).
paramsSubcasesOnly((u) => u.combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const mismatched = t.params.mismatched;

  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const bgl = sourceDevice.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUConst.ShaderStage.VERTEX,
      buffer: {}
    }]

  });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      layout: bgl,
      entries: [
      {
        binding: 0,
        resource: { buffer: t.getUniformBuffer() }
      }]

    });
  }, mismatched);
});

g.test('binding_resources,device_mismatch').
desc(
  `
    Tests createBindGroup cannot be called with various resources created from another device
    Test with two resources to make sure all resources can be validated:
    - resource0 and resource1 from same device
    - resource0 and resource1 from different device

    TODO: test GPUExternalTexture as a resource
    `
).
params((u) =>
u.
combine('entry', [
{ buffer: { type: 'storage' } },
{ sampler: { type: 'filtering' } },
{ texture: { multisampled: false } },
{ storageTexture: { access: 'write-only', format: 'r32float' } },
{ storageTexture: { access: 'read-only', format: 'r32float' } },
{ storageTexture: { access: 'read-write', format: 'r32float' } }]
).
beginSubcases().
combineWithParams([
{ resource0Mismatched: false, resource1Mismatched: false }, //control case
{ resource0Mismatched: true, resource1Mismatched: false },
{ resource0Mismatched: false, resource1Mismatched: true }]
)
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { entry, resource0Mismatched, resource1Mismatched } = t.params;

  const info = bindingTypeInfo(entry);

  const resource0 = resource0Mismatched ?
  t.getDeviceMismatchedBindingResource(info.resource) :
  t.getBindingResource(info.resource);
  const resource1 = resource1Mismatched ?
  t.getDeviceMismatchedBindingResource(info.resource) :
  t.getBindingResource(info.resource);

  const bgl = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: info.validStages,
      ...entry
    },
    {
      binding: 1,
      visibility: info.validStages,
      ...entry
    }]

  });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      layout: bgl,
      entries: [
      {
        binding: 0,
        resource: resource0
      },
      {
        binding: 1,
        resource: resource1
      }]

    });
  }, resource0Mismatched || resource1Mismatched);
});

g.test('storage_texture,usage').
desc(
  `
    Test that the texture usage contains STORAGE_BINDING if the BindGroup entry defines
    storageTexture.
  `
).
params((u) =>
u //
// If usage0 and usage1 are the same, the usage being test is a single usage. Otherwise, it's
// a combined usage.
.combine('usage0', kTextureUsages).
combine('usage1', kTextureUsages)
).
fn((t) => {
  const { usage0, usage1 } = t.params;

  const usage = usage0 | usage1;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      storageTexture: { access: 'write-only', format: 'rgba8unorm' }
    }]

  });

  const texture = t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage
  });

  const isValid = GPUTextureUsage.STORAGE_BINDING & usage;

  const textureView = texture.createView();
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: textureView }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('storage_texture,mip_level_count').
desc(
  `
    Test that the mip level count of the resource of the BindGroup entry as a descriptor is 1 if the
    BindGroup entry defines storageTexture. If the mip level count is not 1, a validation error
    should be generated.
  `
).
params((u) =>
u //
.combine('baseMipLevel', [1, 2]).
combine('mipLevelCount', [1, 2])
).
fn((t) => {
  const { baseMipLevel, mipLevelCount } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      storageTexture: { access: 'write-only', format: 'rgba8unorm' }
    }]

  });

  const MIP_LEVEL_COUNT = 4;
  const texture = t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: 'rgba8unorm',
    usage: GPUTextureUsage.STORAGE_BINDING,
    mipLevelCount: MIP_LEVEL_COUNT
  });

  const textureView = texture.createView({ baseMipLevel, mipLevelCount });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: textureView }],
      layout: bindGroupLayout
    });
  }, mipLevelCount !== 1);
});

g.test('storage_texture,format').
desc(
  `
    Test that the format of the storage texture is equal to resource's descriptor format if the
    BindGroup entry defines storageTexture.
  `
).
params((u) =>
u //
.combine('storageTextureFormat', kStorageTextureFormats).
combine('resourceFormat', kStorageTextureFormats)
).
beforeAllSubcases((t) => {
  const { storageTextureFormat, resourceFormat } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(storageTextureFormat, resourceFormat);
}).
fn((t) => {
  const { storageTextureFormat, resourceFormat } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      storageTexture: { access: 'write-only', format: storageTextureFormat }
    }]

  });

  const texture = t.createTextureTracked({
    size: { width: 16, height: 16, depthOrArrayLayers: 1 },
    format: resourceFormat,
    usage: GPUTextureUsage.STORAGE_BINDING
  });

  const isValid = storageTextureFormat === resourceFormat;
  const textureView = texture.createView({ format: resourceFormat });
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: textureView }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('buffer,usage').
desc(
  `
    Test that the buffer usage contains 'UNIFORM' if the BindGroup entry defines buffer and it's
    type is 'uniform', and the buffer usage contains 'STORAGE' if the BindGroup entry's buffer type
    is 'storage'|read-only-storage'.
  `
).
params((u) =>
u //
.combine('type', kBufferBindingTypes)
// If usage0 and usage1 are the same, the usage being test is a single usage. Otherwise, it's
// a combined usage.
.beginSubcases().
combine('usage0', kBufferUsages).
combine('usage1', kBufferUsages).
unless(
  ({ usage0, usage1 }) =>
  ((usage0 | usage1) & (GPUConst.BufferUsage.MAP_READ | GPUConst.BufferUsage.MAP_WRITE)) !==
  0
)
).
fn((t) => {
  const { type, usage0, usage1 } = t.params;

  const usage = usage0 | usage1;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type }
    }]

  });

  const buffer = t.createBufferTracked({
    size: 4,
    usage
  });

  let isValid = false;
  if (type === 'uniform') {
    isValid = GPUBufferUsage.UNIFORM & usage ? true : false;
  } else if (type === 'storage' || type === 'read-only-storage') {
    isValid = GPUBufferUsage.STORAGE & usage ? true : false;
  }

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer } }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('buffer,resource_offset').
desc(
  `
    Test that the resource.offset of the BindGroup entry is a multiple of limits.
    'minUniformBufferOffsetAlignment|minStorageBufferOffsetAlignment' if the BindGroup entry defines
    buffer and the buffer type is 'uniform|storage|read-only-storage'.
  `
).
params((u) =>
u //
.combine('type', kBufferBindingTypes).
beginSubcases().
combine('offsetAddMult', [
{ add: 0, mult: 0 },
{ add: 0, mult: 0.5 },
{ add: 0, mult: 1.5 },
{ add: 2, mult: 0 }]
)
).
fn((t) => {
  const { type, offsetAddMult } = t.params;
  const minAlignment =
  t.device.limits[
  type === 'uniform' ? 'minUniformBufferOffsetAlignment' : 'minStorageBufferOffsetAlignment'];

  const offset = makeValueTestVariant(minAlignment, offsetAddMult);

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type }
    }]

  });

  const usage = type === 'uniform' ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE;
  const isValid = offset % minAlignment === 0;

  const buffer = t.createBufferTracked({
    size: 1024,
    usage
  });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer, offset } }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('buffer,resource_binding_size').
desc(
  `
    Test that the buffer binding size of the BindGroup entry is equal to or less than limits.
    'maxUniformBufferBindingSize|maxStorageBufferBindingSize' if the BindGroup entry defines
    buffer and the buffer type is 'uniform|storage|read-only-storage'.
  `
).
params((u) =>
u.
combine('type', kBufferBindingTypes).
beginSubcases()
// Test a size of 1 (for uniform buffer) or 4 (for storage and read-only storage buffer)
// then values just within and just above the limit.
.combine('bindingSize', [
{ base: 1, limit: 0 },
{ base: 0, limit: 1 },
{ base: 1, limit: 1 }]
)
).
fn((t) => {
  const {
    type,
    bindingSize: { base, limit }
  } = t.params;
  const mult = type === 'uniform' ? 1 : 4;
  const maxBindingSize =
  t.device.limits[
  type === 'uniform' ? 'maxUniformBufferBindingSize' : 'maxStorageBufferBindingSize'];

  const bindingSize = base * mult + maxBindingSize * limit;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type }
    }]

  });

  const usage = type === 'uniform' ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE;
  const isValid = bindingSize <= maxBindingSize;

  // MAINTENANCE_TODO: Allocating the max size seems likely to fail. Refactor test.
  const buffer = t.createBufferTracked({
    size: maxBindingSize,
    usage
  });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer, size: bindingSize } }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('buffer,effective_buffer_binding_size').
desc(
  `
  Test that the effective buffer binding size of the BindGroup entry must be a multiple of 4 if the
  buffer type is 'storage|read-only-storage', while there is no such restriction on uniform buffers.
`
).
params((u) =>
u.
combine('type', kBufferBindingTypes).
beginSubcases().
combine('offsetMult', [0, 1]).
combine('bufferSizeAddition', [8, 10]).
combine('bindingSize', [undefined, 2, 4, 6])
).
fn((t) => {
  const { type, offsetMult, bufferSizeAddition, bindingSize } = t.params;
  const minAlignment =
  t.device.limits[
  type === 'uniform' ? 'minUniformBufferOffsetAlignment' : 'minStorageBufferOffsetAlignment'];

  const offset = minAlignment * offsetMult;
  const bufferSize = minAlignment + bufferSizeAddition;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      buffer: { type }
    }]

  });

  const effectiveBindingSize = bindingSize ?? bufferSize - offset;
  let usage, isValid;
  if (type === 'uniform') {
    usage = GPUBufferUsage.UNIFORM;
    isValid = true;
  } else {
    usage = GPUBufferUsage.STORAGE;
    isValid = effectiveBindingSize % 4 === 0;
  }

  const buffer = t.createBufferTracked({
    size: bufferSize,
    usage
  });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: { buffer, offset, size: bindingSize } }],
      layout: bindGroupLayout
    });
  }, !isValid);
});

g.test('sampler,device_mismatch').
desc(`Tests createBindGroup cannot be called with a sampler created from another device.`).
paramsSubcasesOnly((u) => u.combine('mismatched', [true, false])).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { mismatched } = t.params;

  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      sampler: { type: 'filtering' }
    }]

  });

  const sampler = sourceDevice.createSampler();
  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: sampler }],
      layout: bindGroupLayout
    });
  }, mismatched);
});

g.test('sampler,compare_function_with_binding_type').
desc(
  `
  Test that the sampler of the BindGroup has a 'compareFunction' value if the sampler type of the
  BindGroupLayout is 'comparison'. Other sampler types should not have 'compare' field in
  the descriptor of the sampler.
  `
).
params((u) =>
u //
.combine('bgType', kSamplerBindingTypes).
beginSubcases().
combine('compareFunction', [undefined, ...kCompareFunctions])
).
fn((t) => {
  const { bgType, compareFunction } = t.params;

  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      sampler: { type: bgType }
    }]

  });

  const isValid =
  bgType === 'comparison' ? compareFunction !== undefined : compareFunction === undefined;

  const sampler = t.device.createSampler({ compare: compareFunction });

  t.expectValidationError(() => {
    t.device.createBindGroup({
      entries: [{ binding: 0, resource: sampler }],
      layout: bindGroupLayout
    });
  }, !isValid);
});