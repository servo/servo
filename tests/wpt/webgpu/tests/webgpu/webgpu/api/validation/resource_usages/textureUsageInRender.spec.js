/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Texture Usages Validation Tests in Render Pass.

Test Coverage:

  - For each combination of two texture usages:
    - For various subresource ranges (different mip levels or array layers) that overlap a given
      subresources or not for color formats:
      - Check that an error is generated when read-write or write-write usages are binding to the
        same texture subresource. Otherwise, no error should be generated. One exception is race
        condition upon two writeonly-storage-texture usages, which is valid.

  - For each combination of two texture usages:
    - For various aspects (all, depth-only, stencil-only) that overlap a given subresources or not
      for depth/stencil formats:
      - Check that an error is generated when read-write or write-write usages are binding to the
        same aspect. Otherwise, no error should be generated.

  - Test combinations of two shader stages:
    - Texture usages in bindings with invisible shader stages should be tracked. Invisible shader
      stages include shader stage with visibility none and compute shader stage in render pass.

  - Tests replaced bindings:
    - Texture usages via bindings replaced by another setBindGroup() upon the same bindGroup index
      in current scope should be tracked.

  - Test texture usages in bundle:
    - Texture usages in bundle should be tracked if that bundle is executed in the current scope.
`;
import { pbool, poptions, params } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  kDepthStencilFormats,
  kDepthStencilFormatInfo,
  kTextureBindingTypes,
  kTextureBindingTypeInfo,
  kShaderStages,
} from '../../../capability_info.js';
import { ValidationTest } from '../validation_test.js';

class TextureUsageTracking extends ValidationTest {
  createTexture(options = {}) {
    const {
      width = 32,
      height = 32,
      arrayLayerCount = 1,
      mipLevelCount = 1,
      sampleCount = 1,
      format = 'rgba8unorm',
      usage = GPUTextureUsage.OUTPUT_ATTACHMENT | GPUTextureUsage.SAMPLED,
    } = options;

    return this.device.createTexture({
      size: { width, height, depth: arrayLayerCount },
      mipLevelCount,
      sampleCount,
      dimension: '2d',
      format,
      usage,
    });
  }

  createBindGroup(index, view, bindingType, bindingTexFormat) {
    return this.device.createBindGroup({
      entries: [{ binding: index, resource: view }],
      layout: this.device.createBindGroupLayout({
        entries: [
          {
            binding: index,
            visibility: GPUShaderStage.FRAGMENT,
            type: bindingType,
            storageTextureFormat: bindingTexFormat,
          },
        ],
      }),
    });
  }
}

export const g = makeTestGroup(TextureUsageTracking);

const BASE_LEVEL = 3;
const BASE_LAYER = 0;
const TOTAL_LEVELS = 6;
const TOTAL_LAYERS = 2;

g.test('subresources_and_binding_types_combination_for_color')
  .params(
    params()
      .combine([
        // Two texture usages are binding to the same texture subresource.
        {
          baseLevel: BASE_LEVEL,
          baseLayer: BASE_LAYER,
          levelCount: 1,
          layerCount: 1,
          _resourceSuccess: false,
        },

        // Two texture usages are binding to different mip levels of the same texture.
        {
          baseLevel: BASE_LEVEL + 1,
          baseLayer: BASE_LAYER,
          levelCount: 1,
          layerCount: 1,
          _resourceSuccess: true,
        },

        // Two texture usages are binding to different array layers of the same texture.
        {
          baseLevel: BASE_LEVEL,
          baseLayer: BASE_LAYER + 1,
          levelCount: 1,
          layerCount: 1,
          _resourceSuccess: true,
        },

        // The second texture usage contains the whole mip chain where the first texture usage is using.
        {
          baseLevel: 0,
          baseLayer: BASE_LAYER,
          levelCount: TOTAL_LEVELS,
          layerCount: 1,
          _resourceSuccess: false,
        },

        // The second texture usage contains the all layers where the first texture usage is using.
        {
          baseLevel: BASE_LEVEL,
          baseLayer: 0,
          levelCount: 1,
          layerCount: TOTAL_LAYERS,
          _resourceSuccess: false,
        },
      ])
      .combine([
        {
          type0: 'sampled-texture',
          type1: 'sampled-texture',
          _usageSuccess: true,
        },

        {
          type0: 'sampled-texture',
          type1: 'readonly-storage-texture',
          _usageSuccess: true,
        },

        {
          type0: 'sampled-texture',
          type1: 'writeonly-storage-texture',
          _usageSuccess: false,
        },

        {
          type0: 'sampled-texture',
          type1: 'render-target',
          _usageSuccess: false,
        },

        {
          type0: 'readonly-storage-texture',
          type1: 'readonly-storage-texture',
          _usageSuccess: true,
        },

        {
          type0: 'readonly-storage-texture',
          type1: 'writeonly-storage-texture',
          _usageSuccess: false,
        },

        {
          type0: 'readonly-storage-texture',
          type1: 'render-target',
          _usageSuccess: false,
        },

        // Race condition upon multiple writable storage texture is valid.
        {
          type0: 'writeonly-storage-texture',
          type1: 'writeonly-storage-texture',
          _usageSuccess: true,
        },

        {
          type0: 'writeonly-storage-texture',
          type1: 'render-target',
          _usageSuccess: false,
        },
      ])
  )
  .fn(async t => {
    const {
      baseLevel,
      baseLayer,
      levelCount,
      layerCount,
      type0,
      type1,
      _usageSuccess,
      _resourceSuccess,
    } = t.params;

    const texture = t.createTexture({
      arrayLayerCount: TOTAL_LAYERS,
      mipLevelCount: TOTAL_LEVELS,
      usage: GPUTextureUsage.SAMPLED | GPUTextureUsage.STORAGE | GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    const view0 = texture.createView({
      baseMipLevel: BASE_LEVEL,
      mipLevelCount: 1,
      baseArrayLayer: BASE_LAYER,
      arrayLayerCount: 1,
    });

    const view1Dimension = layerCount !== 1 ? '2d-array' : '2d';
    const view1 = texture.createView({
      dimension: view1Dimension,
      baseMipLevel: baseLevel,
      mipLevelCount: levelCount,
      baseArrayLayer: baseLayer,
      arrayLayerCount: layerCount,
    });

    // TODO: Add two 'render-target' usages for color attachments.
    const bglEntries = [
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        type: type0,
        storageTextureFormat: type0 === 'sampled-texture' ? undefined : 'rgba8unorm',
      },
    ];

    const bgEntries = [{ binding: 0, resource: view0 }];
    if (type1 !== 'render-target') {
      bglEntries.push({
        binding: 1,
        visibility: GPUShaderStage.FRAGMENT,
        type: type1,
        viewDimension: view1Dimension,
        storageTextureFormat: type1 === 'sampled-texture' ? undefined : 'rgba8unorm',
      });

      bgEntries.push({ binding: 1, resource: view1 });
    }
    const bindGroup = t.device.createBindGroup({
      entries: bgEntries,
      layout: t.device.createBindGroupLayout({ entries: bglEntries }),
    });

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: type1 === 'render-target' ? view1 : t.createTexture().createView(),
          loadValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
          storeOp: 'store',
        },
      ],
    });

    pass.setBindGroup(0, bindGroup);
    pass.endPass();

    const success = _resourceSuccess || _usageSuccess;
    t.expectValidationError(() => {
      encoder.finish();
    }, !success);
  });

g.test('subresources_and_binding_types_combination_for_aspect')
  .params(
    params()
      .combine(poptions('format', kDepthStencilFormats))
      .combine(poptions('aspect0', ['all', 'depth-only', 'stencil-only']))
      .combine(poptions('aspect1', ['all', 'depth-only', 'stencil-only']))
      .unless(
        ({ format, aspect0, aspect1 }) =>
          (aspect0 === 'stencil-only' && !kDepthStencilFormatInfo[format].stencil) ||
          (aspect1 === 'stencil-only' && !kDepthStencilFormatInfo[format].stencil)
      )
      .unless(
        ({ format, aspect0, aspect1 }) =>
          (aspect0 === 'depth-only' && !kDepthStencilFormatInfo[format].depth) ||
          (aspect1 === 'depth-only' && !kDepthStencilFormatInfo[format].depth)
      )
      .combine([
        {
          type0: 'sampled-texture',
          type1: 'sampled-texture',
          _usageSuccess: true,
        },

        {
          type0: 'sampled-texture',
          type1: 'render-target',
          _usageSuccess: false,
        },
      ])
  )
  .fn(async t => {
    const { format, aspect0, aspect1, type0, type1, _usageSuccess } = t.params;

    const texture = t.createTexture({ format });
    const view0 = texture.createView({ aspect: aspect0 });
    const view1 = texture.createView({ aspect: aspect1 });

    const bglEntries = [
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        type: type0,
      },
    ];

    const bgEntries = [{ binding: 0, resource: view0 }];
    if (type1 !== 'render-target') {
      bglEntries.push({
        binding: 1,
        visibility: GPUShaderStage.FRAGMENT,
        type: type1,
      });

      bgEntries.push({ binding: 1, resource: view1 });
    }
    const bindGroup = t.device.createBindGroup({
      entries: bgEntries,
      layout: t.device.createBindGroupLayout({ entries: bglEntries }),
    });

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: t.createTexture().createView(),
          loadValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
          storeOp: 'store',
        },
      ],

      depthStencilAttachment:
        type1 !== 'render-target'
          ? undefined
          : {
              attachment: view1,
              depthStoreOp: 'clear',
              depthLoadValue: 'load',
              stencilStoreOp: 'clear',
              stencilLoadValue: 'load',
            },
    });

    pass.setBindGroup(0, bindGroup);
    pass.endPass();

    const disjointAspects =
      (aspect0 === 'depth-only' && aspect1 === 'stencil-only') ||
      (aspect0 === 'stencil-only' && aspect1 === 'depth-only');
    const success = disjointAspects || _usageSuccess;

    t.expectValidationError(() => {
      encoder.finish();
    }, !success);
  });

g.test('shader_stages_and_visibility')
  .params(
    params()
      .combine(poptions('readVisibility', [0, ...kShaderStages]))
      .combine(poptions('writeVisibility', [0, ...kShaderStages]))
  )
  .fn(async t => {
    const { readVisibility, writeVisibility } = t.params;

    // writeonly-storage-texture binding type is not supported in vertex stage. So, this test
    // uses writeonly-storage-texture binding as writable binding upon the same subresource if
    // vertex stage is not included. Otherwise, it uses output attachment instead.
    const writeHasVertexStage = Boolean(writeVisibility & GPUShaderStage.VERTEX);
    const texUsage = writeHasVertexStage
      ? GPUTextureUsage.SAMPLED | GPUTextureUsage.OUTPUT_ATTACHMENT
      : GPUTextureUsage.SAMPLED | GPUTextureUsage.STORAGE;

    const texture = t.createTexture({ usage: texUsage });
    const view = texture.createView();
    const bglEntries = [{ binding: 0, visibility: readVisibility, type: 'sampled-texture' }];

    const bgEntries = [{ binding: 0, resource: view }];
    if (!writeHasVertexStage) {
      bglEntries.push({
        binding: 1,
        visibility: writeVisibility,
        type: 'writeonly-storage-texture',
        storageTextureFormat: 'rgba8unorm',
      });

      bgEntries.push({ binding: 1, resource: view });
    }
    const bindGroup = t.device.createBindGroup({
      entries: bgEntries,
      layout: t.device.createBindGroupLayout({ entries: bglEntries }),
    });

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: writeHasVertexStage ? view : t.createTexture().createView(),
          loadValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
          storeOp: 'store',
        },
      ],
    });

    pass.setBindGroup(0, bindGroup);
    pass.endPass();

    // Texture usages in bindings with invisible shader stages should be tracked. Invisible shader
    // stages include shader stage with visibility none and compute shader stage in render pass.
    t.expectValidationError(() => {
      encoder.finish();
    });
  });

// We should track the texture usages in bindings which are replaced by another setBindGroup()
// call site upon the same index in the same render pass.
g.test('replaced_binding')
  .params(poptions('bindingType', kTextureBindingTypes))
  .fn(async t => {
    const { bindingType } = t.params;
    const info = kTextureBindingTypeInfo[bindingType];
    const bindingTexFormat = info.resource === 'storageTex' ? 'rgba8unorm' : undefined;

    const sampledView = t.createTexture().createView();
    const sampledStorageView = t
      .createTexture({ usage: GPUTextureUsage.STORAGE | GPUTextureUsage.SAMPLED })
      .createView();

    // Create bindGroup0. It has two bindings. These two bindings use different views/subresources.
    const bglEntries0 = [
      { binding: 0, visibility: GPUShaderStage.FRAGMENT, type: 'sampled-texture' },
      {
        binding: 1,
        visibility: GPUShaderStage.FRAGMENT,
        type: bindingType,
        storageTextureFormat: bindingTexFormat,
      },
    ];

    const bgEntries0 = [
      { binding: 0, resource: sampledView },
      { binding: 1, resource: sampledStorageView },
    ];

    const bindGroup0 = t.device.createBindGroup({
      entries: bgEntries0,
      layout: t.device.createBindGroupLayout({ entries: bglEntries0 }),
    });

    // Create bindGroup1. It has one binding, which use the same view/subresoure of a binding in
    // bindGroup0. So it may or may not conflicts with that binding in bindGroup0.
    const bindGroup1 = t.createBindGroup(0, sampledStorageView, 'sampled-texture', undefined);

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: t.createTexture().createView(),
          loadValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
          storeOp: 'store',
        },
      ],
    });

    // Set bindGroup0 and bindGroup1. bindGroup0 is replaced by bindGroup1 in the current pass.
    // But bindings in bindGroup0 should be tracked too.
    pass.setBindGroup(0, bindGroup0);
    pass.setBindGroup(0, bindGroup1);
    pass.endPass();

    const success = bindingType === 'writeonly-storage-texture' ? false : true;
    t.expectValidationError(() => {
      encoder.finish();
    }, !success);
  });

g.test('bindings_in_bundle')
  .params(
    params()
      .combine(pbool('binding0InBundle'))
      .combine(pbool('binding1InBundle'))
      .combine(poptions('type0', ['render-target', ...kTextureBindingTypes]))
      .combine(poptions('type1', ['render-target', ...kTextureBindingTypes]))
      .unless(
        ({ binding0InBundle, binding1InBundle, type0, type1 }) =>
          // We can't set 'render-target' in bundle, so we need to exclude it from bundle.
          // In addition, if both bindings are non-bundle, there is no need to test it because
          // we have far more comprehensive test cases for that situation in this file.
          (binding0InBundle && type0 === 'render-target') ||
          (binding1InBundle && type1 === 'render-target') ||
          (!binding0InBundle && !binding1InBundle)
      )
  )
  .fn(async t => {
    const { binding0InBundle, binding1InBundle, type0, type1 } = t.params;

    // Two bindings are attached to the same texture view.
    const view = t
      .createTexture({
        usage:
          GPUTextureUsage.OUTPUT_ATTACHMENT | GPUTextureUsage.STORAGE | GPUTextureUsage.SAMPLED,
      })
      .createView();

    const bindGroups = [];
    if (type0 !== 'render-target') {
      const binding0TexFormat = type0 === 'sampled-texture' ? undefined : 'rgba8unorm';
      bindGroups[0] = t.createBindGroup(0, view, type0, binding0TexFormat);
    }
    if (type1 !== 'render-target') {
      const binding1TexFormat = type1 === 'sampled-texture' ? undefined : 'rgba8unorm';
      bindGroups[1] = t.createBindGroup(1, view, type1, binding1TexFormat);
    }

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment:
            // At least one binding is in bundle, which means that its type is not 'render-target'.
            // As a result, only one binding's type is 'render-target' at most.
            type0 === 'render-target' || type1 === 'render-target'
              ? view
              : t.createTexture().createView(),
          loadValue: { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
          storeOp: 'store',
        },
      ],
    });

    const bindingsInBundle = [binding0InBundle, binding1InBundle];
    for (let i = 0; i < 2; i++) {
      // Create a bundle for each bind group if its bindings is required to be in bundle on purpose.
      // Otherwise, call setBindGroup directly in pass if needed (when its binding is not
      // 'render-target').
      if (bindingsInBundle[i]) {
        const bundleEncoder = t.device.createRenderBundleEncoder({
          colorFormats: ['rgba8unorm'],
        });

        bundleEncoder.setBindGroup(i, bindGroups[i]);
        const bundleInPass = bundleEncoder.finish();
        pass.executeBundles([bundleInPass]);
      } else if (bindGroups[i] !== undefined) {
        pass.setBindGroup(i, bindGroups[i]);
      }
    }

    pass.endPass();

    let success = false;
    if (
      (type0 === 'sampled-texture' || type0 === 'readonly-storage-texture') &&
      (type1 === 'sampled-texture' || type1 === 'readonly-storage-texture')
    ) {
      success = true;
    }

    if (type0 === 'writeonly-storage-texture' && type1 === 'writeonly-storage-texture') {
      success = true;
    }

    // Resource usages in bundle should be tracked. And validation error should be reported
    // if needed.
    t.expectValidationError(() => {
      encoder.finish();
    }, !success);
  });
