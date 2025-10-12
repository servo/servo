/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for capability checking for the 'texture-formats-tier2' feature.

Test that enabling texture-formats-tier2 also enables rg11b10ufloat-renderable and texture-formats-tier1

Tests that abilities enabled by 'texture-formats-tier2' correctly generate validation errors
when the feature is not enabled. This includes:
- read-write stoorage access formats gaining this capability.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kTextureFormatsTier2EnablesStorageReadWrite } from '../../../../format_info.js';
import { UniqueFeaturesOrLimitsGPUTest } from '../../../../gpu_test.js';
import * as vtu from '../../validation_test_utils.js';

export const g = makeTestGroup(UniqueFeaturesOrLimitsGPUTest);

g.test('enables_rg11b10ufloat_renderable_and_texture_formats_tier1').
desc(
  `
  Test that enabling texture-formats-tier2 also enables rg11b10ufloat-renderable and  texture-formats-tier1
  `
).
beforeAllSubcases((t) => t.selectDeviceOrSkipTestCase('texture-formats-tier2')).
fn((t) => {
  t.expect(() => t.device.features.has('rg11b10ufloat-renderable'));
  t.expect(() => t.device.features.has('texture-formats-tier1'));
});

g.test('bind_group_layout,storage_binding_read_write_access').
desc(
  `
  Test a bindGroupLayout with access 'read-write' and a format enabled by
  'texture-formats-tier2' fails if the feature is not enabled and succeeds
  if it is enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatsTier2EnablesStorageReadWrite).
combine('enable_feature', [true, false])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier2');
  }
}).
fn((t) => {
  const { format, enable_feature } = t.params;

  t.expectValidationError(() => {
    t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: {
          access: 'read-write',
          format
        }
      }]

    });
  }, !enable_feature);
});

g.test('pipeline_auto_layout,storage_texture').
desc(
  `
  Test creating a pipeline with auto layout with a storage texture binding format enabled by
  'texture-formats-tier2' fails if the feature is not enabled.
  `
).
params((u) =>
u.
combine('format', kTextureFormatsTier2EnablesStorageReadWrite).
combine('enable_feature', [true, false]).
beginSubcases().
combine('isAsync', [false, true]).
combine('type', ['compute', 'render'])
).
beforeAllSubcases((t) => {
  const { enable_feature } = t.params;
  if (enable_feature) {
    t.selectDeviceOrSkipTestCase('texture-formats-tier2');
  }
}).
fn((t) => {
  const { format, enable_feature, isAsync, type } = t.params;

  const code = `
      @group(0) @binding(0) var tex1d: texture_storage_1d<${format}, read_write>;
      @group(0) @binding(1) var tex2d: texture_storage_1d<${format}, read_write>;
      @group(0) @binding(2) var tex3d: texture_storage_1d<${format}, read_write>;

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