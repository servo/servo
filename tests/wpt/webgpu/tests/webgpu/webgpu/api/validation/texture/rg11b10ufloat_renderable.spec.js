/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for capabilities added by rg11b10ufloat-renderable flag.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUConst } from '../../../constants.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('create_texture').
desc(
  `
Test that it is valid to create rg11b10ufloat texture with RENDER_ATTACHMENT usage and/or
sampleCount > 1, iff rg11b10ufloat-renderable feature is enabled.
Note, the createTexture tests cover these validation cases where this feature is not enabled.
`
).
params((u) => u.combine('sampleCount', [1, 4])).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('rg11b10ufloat-renderable');
}).
fn((t) => {
  const { sampleCount } = t.params;
  const descriptor = {
    size: [1, 1, 1],
    format: 'rg11b10ufloat',
    sampleCount,
    usage: GPUConst.TextureUsage.RENDER_ATTACHMENT
  };
  t.device.createTexture(descriptor);
});

g.test('begin_render_pass_single_sampled').
desc(
  `
Test that it is valid to begin render pass with rg11b10ufloat texture format
iff rg11b10ufloat-renderable feature is enabled. Single sampled case.
`
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('rg11b10ufloat-renderable');
}).
fn((t) => {
  const texture = t.device.createTexture({
    size: [1, 1, 1],
    format: 'rg11b10ufloat',
    sampleCount: 1,
    usage: GPUConst.TextureUsage.RENDER_ATTACHMENT
  });
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: texture.createView(),
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.end();
  encoder.finish();
});

g.test('begin_render_pass_msaa_and_resolve').
desc(
  `
Test that it is valid to begin render pass with rg11b10ufloat texture format
iff rg11b10ufloat-renderable feature is enabled. MSAA and resolve case.
`
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('rg11b10ufloat-renderable');
}).
fn((t) => {
  const renderTexture = t.device.createTexture({
    size: [1, 1, 1],
    format: 'rg11b10ufloat',
    sampleCount: 4,
    usage: GPUConst.TextureUsage.RENDER_ATTACHMENT
  });
  const resolveTexture = t.device.createTexture({
    size: [1, 1, 1],
    format: 'rg11b10ufloat',
    sampleCount: 1,
    usage: GPUConst.TextureUsage.RENDER_ATTACHMENT
  });
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTexture.createView(),
      resolveTarget: resolveTexture.createView(),
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.end();
  encoder.finish();
});

g.test('begin_render_bundle_encoder').
desc(
  `
Test that it is valid to begin render bundle encoder with rg11b10ufloat texture
format iff rg11b10ufloat-renderable feature is enabled.
`
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('rg11b10ufloat-renderable');
}).
fn((t) => {
  t.device.createRenderBundleEncoder({
    colorFormats: ['rg11b10ufloat']
  });
});

g.test('create_render_pipeline').
desc(
  `
Test that it is valid to create render pipeline with rg11b10ufloat texture format
in descriptor.fragment.targets iff rg11b10ufloat-renderable feature is enabled.
`
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase('rg11b10ufloat-renderable');
}).
fn((t) => {
  t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: t.getNoOpShaderCode('VERTEX')
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: t.getNoOpShaderCode('FRAGMENT')
      }),
      entryPoint: 'main',
      targets: [{ format: 'rg11b10ufloat', writeMask: 0 }]
    },
    primitive: { topology: 'triangle-list' }
  });
});