/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests execution of render bundles.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kDepthStencilFormats } from '../../../format_info.js';
import { ValidationTest } from '../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('empty_bundle_list').
desc(
  `
    Test that it is valid to execute an empty list of render bundles
    `
).
fn((t) => {
  const encoder = t.createEncoder('render pass');
  encoder.encoder.executeBundles([]);
  encoder.validateFinish(true);
});

g.test('device_mismatch').
desc(
  `
    Tests executeBundles cannot be called with render bundles created from another device
    Test with two bundles to make sure all bundles can be validated:
    - bundle0 and bundle1 from same device
    - bundle0 and bundle1 from different device
    `
).
paramsSubcasesOnly([
{ bundle0Mismatched: false, bundle1Mismatched: false }, // control case
{ bundle0Mismatched: true, bundle1Mismatched: false },
{ bundle0Mismatched: false, bundle1Mismatched: true }]
).
beforeAllSubcases((t) => {
  t.selectMismatchedDeviceOrSkipTestCase(undefined);
}).
fn((t) => {
  const { bundle0Mismatched, bundle1Mismatched } = t.params;

  const descriptor = {
    colorFormats: ['rgba8unorm']
  };

  const bundle0Device = bundle0Mismatched ? t.mismatchedDevice : t.device;
  const bundle0 = bundle0Device.createRenderBundleEncoder(descriptor).finish();

  const bundle1Device = bundle1Mismatched ? t.mismatchedDevice : t.device;
  const bundle1 = bundle1Device.createRenderBundleEncoder(descriptor).finish();

  const encoder = t.createEncoder('render pass');
  encoder.encoder.executeBundles([bundle0, bundle1]);

  encoder.validateFinish(!(bundle0Mismatched || bundle1Mismatched));
});

g.test('color_formats_mismatch').
desc(
  `
    Tests executeBundles cannot be called with render bundles that do match the colorFormats of the
    render pass. This includes:
    - formats don't match
    - formats match but are in a different order
    - formats match but there is a different count
    `
).
params((u) =>
u.combineWithParams([
{
  bundleFormats: ['bgra8unorm', 'rg8unorm'],
  passFormats: ['bgra8unorm', 'rg8unorm'],
  _compatible: true
}, // control case
{
  bundleFormats: ['bgra8unorm', 'rg8unorm'],
  passFormats: ['bgra8unorm', 'bgra8unorm'],
  _compatible: false
},
{
  bundleFormats: ['bgra8unorm', 'rg8unorm'],
  passFormats: ['rg8unorm', 'bgra8unorm'],
  _compatible: false
},
{
  bundleFormats: ['bgra8unorm', 'rg8unorm', 'rgba8unorm'],
  passFormats: ['rg8unorm', 'bgra8unorm'],
  _compatible: false
},
{
  bundleFormats: ['bgra8unorm', 'rg8unorm'],
  passFormats: ['rg8unorm', 'bgra8unorm', 'rgba8unorm'],
  _compatible: false
}]
)
).
fn((t) => {
  const { bundleFormats, passFormats, _compatible } = t.params;

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: bundleFormats
  });
  const bundle = bundleEncoder.finish();

  const encoder = t.createEncoder('render pass', {
    attachmentInfo: {
      colorFormats: passFormats
    }
  });
  encoder.encoder.executeBundles([bundle]);

  encoder.validateFinish(_compatible);
});

g.test('depth_stencil_formats_mismatch').
desc(
  `
    Tests executeBundles cannot be called with render bundles that do match the depthStencil of the
    render pass. This includes:
    - formats don't match
    - formats have matching depth or stencil aspects, but other aspects are missing
    `
).
params((u) =>
u.combineWithParams([
{ bundleFormat: 'depth24plus', passFormat: 'depth24plus' }, // control case
{ bundleFormat: 'depth24plus', passFormat: 'depth16unorm' },
{ bundleFormat: 'depth24plus', passFormat: 'depth24plus-stencil8' },
{ bundleFormat: 'stencil8', passFormat: 'depth24plus-stencil8' }]
)
).
beforeAllSubcases((t) => {
  const { bundleFormat, passFormat } = t.params;
  t.selectDeviceForTextureFormatOrSkipTestCase([bundleFormat, passFormat]);
}).
fn((t) => {
  const { bundleFormat, passFormat } = t.params;
  const compatible = bundleFormat === passFormat;

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: [],
    depthStencilFormat: bundleFormat
  });
  const bundle = bundleEncoder.finish();

  const encoder = t.createEncoder('render pass', {
    attachmentInfo: {
      colorFormats: [],
      depthStencilFormat: passFormat
    }
  });
  encoder.encoder.executeBundles([bundle]);

  encoder.validateFinish(compatible);
});

g.test('depth_stencil_readonly_mismatch').
desc(
  `
    Tests executeBundles cannot be called with render bundles that do match the depthStencil
    readonly state of the render pass.
    `
).
params((u) =>
u.
combine('depthStencilFormat', kDepthStencilFormats).
beginSubcases().
combine('bundleDepthReadOnly', [false, true]).
combine('bundleStencilReadOnly', [false, true]).
combine('passDepthReadOnly', [false, true]).
combine('passStencilReadOnly', [false, true])
).
beforeAllSubcases((t) => {
  t.selectDeviceForTextureFormatOrSkipTestCase(t.params.depthStencilFormat);
}).
fn((t) => {
  const {
    depthStencilFormat,
    bundleDepthReadOnly,
    bundleStencilReadOnly,
    passDepthReadOnly,
    passStencilReadOnly
  } = t.params;

  const compatible =
  (!passDepthReadOnly || bundleDepthReadOnly === passDepthReadOnly) && (
  !passStencilReadOnly || bundleStencilReadOnly === passStencilReadOnly);

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: [],
    depthStencilFormat,
    depthReadOnly: bundleDepthReadOnly,
    stencilReadOnly: bundleStencilReadOnly
  });
  const bundle = bundleEncoder.finish();

  const encoder = t.createEncoder('render pass', {
    attachmentInfo: {
      colorFormats: [],
      depthStencilFormat,
      depthReadOnly: passDepthReadOnly,
      stencilReadOnly: passStencilReadOnly
    }
  });
  encoder.encoder.executeBundles([bundle]);

  encoder.validateFinish(compatible);
});

g.test('sample_count_mismatch').
desc(
  `
    Tests executeBundles cannot be called with render bundles that do match the sampleCount of the
    render pass.
    `
).
params((u) =>
u.combineWithParams([
{ bundleSamples: 1, passSamples: 1 }, // control case
{ bundleSamples: 4, passSamples: 4 }, // control case
{ bundleFormat: 4, passFormat: 1 },
{ bundleFormat: 1, passFormat: 4 }]
)
).
fn((t) => {
  const { bundleSamples, passSamples } = t.params;

  const compatible = bundleSamples === passSamples;

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['bgra8unorm'],
    sampleCount: bundleSamples
  });
  const bundle = bundleEncoder.finish();

  const encoder = t.createEncoder('render pass', {
    attachmentInfo: {
      colorFormats: ['bgra8unorm'],
      sampleCount: passSamples
    }
  });
  encoder.encoder.executeBundles([bundle]);

  encoder.validateFinish(compatible);
});