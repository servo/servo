/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that, in compat mode, the dimension of a view is compatible with a texture's textureBindingViewDimension.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kTextureDimensions, kTextureViewDimensions } from '../../../capability_info.js';
import {
  effectiveViewDimensionForTexture,
  getTextureDimensionFromView } from
'../../../util/texture/base.js';
import { CompatibilityTest } from '../../compatibility_test.js';

export const g = makeTestGroup(CompatibilityTest);

function isTextureBindingViewDimensionCompatibleWithDimension(
dimension = '2d',
textureBindingViewDimension = '2d')
{
  return getTextureDimensionFromView(textureBindingViewDimension) === dimension;
}

function isValidViewDimensionForDimension(
dimension,
depthOrArrayLayers,
viewDimension)
{
  if (viewDimension === undefined) {
    return true;
  }

  switch (dimension) {
    case '1d':
      return viewDimension === '1d';
    case '2d':
    case undefined:
      switch (viewDimension) {
        case undefined:
        case '2d':
        case '2d-array':
          return true;
        case 'cube':
          return depthOrArrayLayers === 6;
        case 'cube-array':
          return depthOrArrayLayers % 6 === 0;
        default:
          return false;
      }
      break;
    case '3d':
      return viewDimension === '3d';
  }
}

function isValidDimensionForDepthOrArrayLayers(
dimension,
depthOrArrayLayers)
{
  switch (dimension) {
    case '1d':
      return depthOrArrayLayers === 1;
    default:
      return true;
  }
}

function isValidViewDimensionForDepthOrArrayLayers(
viewDimension,
depthOrArrayLayers)
{
  switch (viewDimension) {
    case '2d':
      return depthOrArrayLayers === 1;
    case 'cube':
      return depthOrArrayLayers === 6;
    case 'cube-array':
      return depthOrArrayLayers % 6 === 0;
    default:
      return true;
  }
  return viewDimension === 'cube';
}

function getEffectiveTextureBindingViewDimension(
dimension,
depthOrArrayLayers,
textureBindingViewDimension)
{
  if (textureBindingViewDimension) {
    return textureBindingViewDimension;
  }

  switch (dimension) {
    case '1d':
      return '1d';
    case '2d':
    case undefined:
      return depthOrArrayLayers > 1 ? '2d-array' : '2d';
      break;
    case '3d':
      return '3d';
  }
}

g.test('viewDimension_matches_textureBindingViewDimension').
desc(
  `
    Tests that, in compat mode, the dimension of a view is compatible with a texture's textureBindingViewDimension
    when used as a TEXTURE_BINDING.
    `
).
params((u) =>
u //
.combine('dimension', [...kTextureDimensions, undefined]).
combine('textureBindingViewDimension', [...kTextureViewDimensions, undefined]).
combine('viewDimension', [...kTextureViewDimensions, undefined]).
combine('depthOrArrayLayers', [1, 2, 6]).
filter(
  ({ dimension, textureBindingViewDimension, depthOrArrayLayers, viewDimension }) =>
  textureBindingViewDimension !== 'cube-array' &&
  viewDimension !== 'cube-array' &&
  isTextureBindingViewDimensionCompatibleWithDimension(
    dimension,
    textureBindingViewDimension
  ) &&
  isValidViewDimensionForDimension(dimension, depthOrArrayLayers, viewDimension) &&
  isValidViewDimensionForDepthOrArrayLayers(
    textureBindingViewDimension,
    depthOrArrayLayers
  ) &&
  isValidDimensionForDepthOrArrayLayers(dimension, depthOrArrayLayers)
)
).
fn((t) => {
  const { dimension, textureBindingViewDimension, viewDimension, depthOrArrayLayers } = t.params;

  const texture = t.createTextureTracked({
    size: [1, 1, depthOrArrayLayers],
    format: 'rgba8unorm',
    usage: GPUTextureUsage.TEXTURE_BINDING,
    ...(dimension && { dimension }),
    ...(textureBindingViewDimension && { textureBindingViewDimension })
  }); // MAINTENANCE_TODO: remove cast once textureBindingViewDimension is added to IDL

  const effectiveTextureBindingViewDimension = getEffectiveTextureBindingViewDimension(
    dimension,
    texture.depthOrArrayLayers,
    textureBindingViewDimension
  );

  const effectiveViewDimension = getEffectiveTextureBindingViewDimension(
    dimension,
    texture.depthOrArrayLayers,
    viewDimension
  );

  const layout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      texture: {
        viewDimension: effectiveViewDimensionForTexture(texture, viewDimension)
      }
    }]

  });

  const resource = texture.createView({ dimension: viewDimension });
  const shouldError = effectiveTextureBindingViewDimension !== effectiveViewDimension;

  t.expectValidationErrorInCompatibilityMode(() => {
    t.device.createBindGroup({
      layout,
      entries: [{ binding: 0, resource }]
    });
  }, shouldError);
});