/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for capability checking for the 'texture-formats-tier1' feature.

Test that enabling texture-formats-tier1 also enables rg11b10ufloat-renderable

Tests that abilities enabled by 'texture-formats-tier1' correctly generate validation errors
when the feature is not enabled. This includes:
- RENDER_ATTACHMENT usage for formats gaining this capability.
- Multisample usage for formats gaining this capability.
- Blendability for formats gaining this capability.
- Resolvability for formats gaining this capability.
- STORAGE_BINDING usage for formats gaining this capability.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { hasFeature } from '../../../../../common/util/util.js';
import {
  kTextureFormatTier1AllowsRenderAttachmentBlendableMultisample,
  kTextureFormatTier1ThrowsWhenNotEnabled,
  kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly,
  kTextureFormatTier1AllowsResolve } from
'../../../../format_info.js';
import { UniqueFeaturesOrLimitsGPUTest } from '../../../../gpu_test.js';
import * as vtu from '../../validation_test_utils.js';

export const g = makeTestGroup(UniqueFeaturesOrLimitsGPUTest);

g.test('enables_rg11b10ufloat_renderable').
desc(
  `
  Test that enabling texture-formats-tier1 also enables rg11b10ufloat-renderable
  `
).
beforeAllSubcases((t) => t.selectDeviceOrSkipTestCase('texture-formats-tier1')).
fn((t) => {
  t.expect(() => hasFeature(t.device.features, 'rg11b10ufloat-renderable'));
});

g.test('texture_usage,render_attachment').
desc(
  `
  Test creating a texture with RENDER_ATTACHMENT usage and a format enabled by
  'texture-formats-tier1' fails if the feature is not enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatTier1AllowsRenderAttachmentBlendableMultisample).
combine('enable_feature', [true, false])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { format, enable_feature } = t.params;

  t.expectValidationErrorOrException(
    () => {
      t.createTextureTracked({
        format,
        size: [1, 1, 1],
        usage: GPUTextureUsage.RENDER_ATTACHMENT
      });
    },
    !enable_feature,
    kTextureFormatTier1ThrowsWhenNotEnabled.includes(format)
  );
});

g.test('texture_usage,multisample').
desc(
  `
  Test creating a multisampled texture with a format enabled by
  'texture-formats-tier1' fails if the feature is not enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatTier1AllowsRenderAttachmentBlendableMultisample).
combine('enable_feature', [true, false])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { format, enable_feature } = t.params;

  t.expectValidationErrorOrException(
    () => {
      t.createTextureTracked({
        format,
        size: [1, 1, 1],
        usage: GPUTextureUsage.RENDER_ATTACHMENT,
        sampleCount: 4
      });
    },
    !enable_feature,
    kTextureFormatTier1ThrowsWhenNotEnabled.includes(format)
  );
});

g.test('texture_usage,storage_binding').
desc(
  `
  Test creating a texture with STORAGE_BINDING usage and a format enabled by
  'texture-formats-tier1' fails if the feature is not enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly).
combine('enable_feature', [true, false])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { format, enable_feature } = t.params;

  t.expectValidationErrorOrException(
    () => {
      t.createTextureTracked({
        format,
        size: [1, 1, 1],
        usage: GPUTextureUsage.STORAGE_BINDING
      });
    },
    !enable_feature,
    kTextureFormatTier1ThrowsWhenNotEnabled.includes(format)
  );
});

g.test('render_pipeline,color_target').
desc(
  `
  Test creating a render pipeline with a color target format enabled by
  'texture-formats-tier1' fails if the feature is not enabled.
  This covers RENDER_ATTACHMENT, blendable, and multisample capabilities.

  Note: it's not clear it's possible to check blendable and multisample
  as most likely there will be an error for RENDER_ATTACHMENT first.
  `
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', [
'rgba8unorm',
...kTextureFormatTier1AllowsRenderAttachmentBlendableMultisample]
).
combine('enable_feature', [true, false]).
combine('check', ['RENDER_ATTACHMENT', 'blendable', 'multisample'])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { isAsync, format, enable_feature, check } = t.params;

  const pipelineDescriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex
            fn main()-> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @fragment
            fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format }]
    }
  };
  const target0 = pipelineDescriptor.fragment.targets[0];

  if (check === 'multisample') {
    pipelineDescriptor.multisample = { count: 4 };
  }

  if (check === 'blendable') {
    target0.blend = {
      color: { operation: 'add', srcFactor: 'one', dstFactor: 'zero' },
      alpha: { operation: 'add', srcFactor: 'one', dstFactor: 'zero' }
    };
  }

  vtu.doCreateRenderPipelineTest(
    t,
    isAsync,
    enable_feature || format === 'rgba8unorm',
    pipelineDescriptor,
    kTextureFormatTier1ThrowsWhenNotEnabled.includes(format) ? 'TypeError' : 'GPUPipelineError'
  );
});

g.test('render_pass,resolvable').
desc(
  `
  Test creating a render pass with resolve with a color target format enabled by
  'texture-formats-tier1' success if the feature is enabled.

  Note: It's not possible to test the failure case (feature disabled).
  Because you won't be able to create a render pipeline that passes validation which
  you need before you can create a render pass that resolves.
  `
).
params((u) =>
u.combine('format', kTextureFormatTier1AllowsResolve).combine('enable_feature', [true])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { format } = t.params;

  const size = [1, 1, 1];
  const sampleCount = 4;

  const msaaTexture = t.createTextureTracked({
    format,
    size,
    sampleCount,
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  });

  const resolveTexture = t.createTextureTracked({
    format,
    size,
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  });

  const descriptor = {
    colorAttachments: [
    {
      view: msaaTexture.createView(),
      resolveTarget: resolveTexture.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }]

  };

  const encoder = t.device.createCommandEncoder();
  encoder.beginRenderPass(descriptor);
});

g.test('bind_group_layout,storage_texture').
desc(
  `
  Test creating a bind group layout with a storage texture binding format enabled by
  'texture-formats-tier1' fails if the feature is not enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly).
combine('access', ['read-only', 'write-only']) // Tier1 enables read-only/write-only for these
.combine('enable_feature', [true, false])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { format, access, enable_feature } = t.params;

  t.expectValidationErrorOrException(
    () => {
      t.device.createBindGroupLayout({
        entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.COMPUTE,
          storageTexture: {
            format,
            access
          }
        }]

      });
    },
    !enable_feature,
    kTextureFormatTier1ThrowsWhenNotEnabled.includes(format)
  );
});

g.test('pipeline_auto_layout,storage_texture').
desc(
  `
  Test creating a pipeline with auto layout with a storage texture binding format enabled by
  'texture-formats-tier1' fails if the feature is not enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly).
combine('access', ['read', 'write']) // Tier1 enables read-only/write-only for these
.combine('enable_feature', [true, false]).
beginSubcases().
combine('isAsync', [false, true]).
combine('type', ['compute', 'render'])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier1');
  }
}).
fn((t) => {
  const { format, access, enable_feature, isAsync, type } = t.params;

  const code = `
      @group(0) @binding(0) var tex1d: texture_storage_1d<${format}, ${access}>;
      @group(0) @binding(1) var tex2d: texture_storage_1d<${format}, ${access}>;
      @group(0) @binding(2) var tex3d: texture_storage_1d<${format}, ${access}>;

      fn useTextures() {
        _ = tex1d;
        _ = tex2d;
        _ = tex3d;
      }

      @compute @workgroup_size(1) fn cs() {
        useTextures();
      }

      @vertex fn vs() -> @builtin(position) vec4f {
        return vec4f(0);
      }
      @fragment fn fs() -> @location(0) vec4f {
        useTextures();
        return vec4f(0);
      }
    `;

  const module = t.device.createShaderModule({ code });

  if (type === 'compute') {
    const descriptor = {
      layout: 'auto',
      compute: { module }
    };
    vtu.doCreateComputePipelineTest(t, isAsync, enable_feature, descriptor);
  } else {
    const descriptor = {
      layout: 'auto',
      vertex: { module },
      fragment: { module, targets: [{ format: 'rgba8unorm' }] }
    };
    vtu.doCreateRenderPipelineTest(t, isAsync, enable_feature, descriptor);
  }
});