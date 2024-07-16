/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that you can not use bgra8unorm-srgb in compat mode.
Tests that textureBindingViewDimension must compatible with texture dimension
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { kTextureDimensions, kTextureViewDimensions } from '../../../../capability_info.js';
import {
  kColorTextureFormats,
  kCompatModeUnsupportedStorageTextureFormats,
  kTextureFormatInfo } from
'../../../../format_info.js';
import { getTextureDimensionFromView } from '../../../../util/texture/base.js';
import { CompatibilityTest } from '../../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

g.test('unsupportedTextureFormats').
desc(`Tests that you can not create a bgra8unorm-srgb texture in compat mode.`).
fn((t) => {
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.createTextureTracked({
      size: [1, 1, 1],
      format: 'bgra8unorm-srgb',
      usage: GPUTextureUsage.TEXTURE_BINDING
    }),
    true
  );
});

g.test('unsupportedTextureViewFormats').
desc(
  `Tests that you can not create a bgra8unorm texture with a bgra8unorm-srgb viewFormat in compat mode.`
).
fn((t) => {
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.createTextureTracked({
      size: [1, 1, 1],
      format: 'bgra8unorm',
      viewFormats: ['bgra8unorm-srgb'],
      usage: GPUTextureUsage.TEXTURE_BINDING
    }),
    true
  );
});

g.test('invalidTextureBindingViewDimension').
desc(
  `Tests that you can not specify a textureBindingViewDimension that is incompatible with the texture's dimension.`
).
params((u) =>
u //
.combine('dimension', kTextureDimensions).
combine('textureBindingViewDimension', kTextureViewDimensions)
).
fn((t) => {
  const { dimension, textureBindingViewDimension } = t.params;
  const depthOrArrayLayers =
  dimension === '1d' ||
  textureBindingViewDimension === '1d' ||
  textureBindingViewDimension === '2d' ?
  1 :
  6;
  const shouldError = getTextureDimensionFromView(textureBindingViewDimension) !== dimension;
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () => {
      t.createTextureTracked({
        size: [1, 1, depthOrArrayLayers],
        format: 'rgba8unorm',
        usage: GPUTextureUsage.TEXTURE_BINDING,
        dimension,
        textureBindingViewDimension
      }); // MAINTENANCE_TODO: remove cast once textureBindingViewDimension is added to IDL
    },
    shouldError
  );
});

g.test('depthOrArrayLayers_incompatible_with_textureBindingViewDimension').
desc(
  `Tests
    * if textureBindingViewDimension is '2d' then depthOrArrayLayers must be 1
    * if textureBindingViewDimension is 'cube' then depthOrArrayLayers must be 6
    `
).
params((u) =>
u //
.combine('textureBindingViewDimension', ['2d', 'cube']).
combine('depthOrArrayLayers', [1, 3, 6, 12])
).
fn((t) => {
  const { textureBindingViewDimension, depthOrArrayLayers } = t.params;
  const shouldError =
  textureBindingViewDimension === '2d' && depthOrArrayLayers !== 1 ||
  textureBindingViewDimension === 'cube' && depthOrArrayLayers !== 6;
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () => {
      t.createTextureTracked({
        size: [1, 1, depthOrArrayLayers],
        format: 'rgba8unorm',
        usage: GPUTextureUsage.TEXTURE_BINDING,
        textureBindingViewDimension
      }); // MAINTENANCE_TODO: remove cast once textureBindingViewDimension is added to IDL
    },
    shouldError
  );
});

g.test('format_reinterpretation').
desc(
  `
    Tests that you can not request different view formats when creating a texture.
    For example, rgba8unorm can not be viewed as rgba8unorm-srgb
  `
).
params((u) =>
u //
.combine('format', kColorTextureFormats).
filter(
  ({ format }) =>
  !!kTextureFormatInfo[format].baseFormat &&
  kTextureFormatInfo[format].baseFormat !== format
)
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];

  const formatPairs = [
  { format, viewFormats: [info.baseFormat] },
  { format: info.baseFormat, viewFormats: [format] },
  { format, viewFormats: [format, info.baseFormat] },
  { format: info.baseFormat, viewFormats: [format, info.baseFormat] }];

  for (const { format, viewFormats } of formatPairs) {
    t.expectGPUErrorInCompatibilityMode(
      'validation',
      () => {
        t.createTextureTracked({
          size: [info.blockWidth, info.blockHeight],
          format,
          viewFormats,
          usage: GPUTextureUsage.TEXTURE_BINDING
        });
      },
      true
    );
  }
});

g.test('unsupportedStorageTextureFormats').
desc(`Tests that you can not create unsupported storage texture formats in compat mode.`).
params((u) => u.combine('format', kCompatModeUnsupportedStorageTextureFormats)).
fn((t) => {
  const { format } = t.params;
  t.expectGPUErrorInCompatibilityMode(
    'validation',
    () =>
    t.createTextureTracked({
      size: [1, 1, 1],
      format,
      usage: GPUTextureUsage.STORAGE_BINDING
    }),
    true
  );
});