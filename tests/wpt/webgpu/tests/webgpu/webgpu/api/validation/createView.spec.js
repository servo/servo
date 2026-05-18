/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `createView validation tests.`;import { AllFeaturesMaxLimitsGPUTest, kResourceStates } from '../.././gpu_test.js';
import { kUnitCaseParamsBuilder } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { unreachable } from '../../../common/util/util.js';
import {
  isValidTextureUsageCombination,
  kTextureAspects,
  kTextureDimensions,
  kTextureUsages,
  kTextureViewDimensions } from
'../../capability_info.js';
import { GPUConst } from '../../constants.js';
import {
  kAllTextureFormats,
  kFeaturesForFormats,
  filterFormatsByFeature,
  textureFormatsAreViewCompatible,
  isDepthTextureFormat,
  isTextureFormatUsableWithStorageAccessMode,
  isTextureFormatColorRenderable,
  isColorTextureFormat,
  isStencilTextureFormat,
  getBlockInfoForTextureFormat,
  isTextureFormatPossiblyUsableAsRenderAttachment } from
'../../format_info.js';
import {
  getTextureDimensionFromView,
  reifyTextureViewDescriptor,
  viewDimensionsForTextureDimension } from
'../../util/texture/base.js';
import { reifyExtent3D } from '../../util/unions.js';

import * as vtu from './validation_test_utils.js';

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

const kLevels = 6;

g.test('format').
desc(
  `Views must have the view format compatible with the base texture, for all {texture format}x{view format}.`
).
params((u) =>
u.
combine('textureFormatFeature', kFeaturesForFormats).
combine('viewFormatFeature', kFeaturesForFormats).
beginSubcases().
expand('textureFormat', ({ textureFormatFeature }) =>
filterFormatsByFeature(textureFormatFeature, kAllTextureFormats)
).
expand('viewFormat', ({ viewFormatFeature }) =>
filterFormatsByFeature(viewFormatFeature, [undefined, ...kAllTextureFormats])
).
combine('useViewFormatList', [false, true])
).
fn((t) => {
  const { textureFormat, viewFormat, useViewFormatList } = t.params;
  const { blockWidth, blockHeight } = getBlockInfoForTextureFormat(textureFormat);

  t.skipIfTextureFormatNotSupported(textureFormat, viewFormat);

  const compatible =
  viewFormat === undefined ||
  textureFormatsAreViewCompatible(t.device.features, textureFormat, viewFormat);

  const texture = t.createTextureTracked({
    format: textureFormat,
    size: [blockWidth, blockHeight],
    usage: GPUTextureUsage.TEXTURE_BINDING,

    // This is a test of createView, not createTexture. Don't pass viewFormats here that
    // are not compatible, as that is tested in createTexture.spec.ts.
    viewFormats:
    useViewFormatList && compatible && viewFormat !== undefined ? [viewFormat] : undefined
  });

  // Successful if there is no view format, no reinterpretation was required, or the formats are compatible
  // and is was specified in the viewFormats list.
  const success =
  viewFormat === undefined || viewFormat === textureFormat || compatible && useViewFormatList;
  t.expectValidationError(() => {
    texture.createView({ format: viewFormat });
  }, !success);
});

g.test('dimension').
desc(
  `For all {texture dimension}, {view dimension}, test that they must be compatible:
  - 1d -> 1d
  - 2d -> 2d, 2d-array, cube, or cube-array
  - 3d -> 3d`
).
params((u) =>
u.
combine('textureDimension', kTextureDimensions).
combine('viewDimension', [...kTextureViewDimensions, undefined])
).
fn((t) => {
  const { textureDimension, viewDimension } = t.params;
  t.skipIfTextureViewDimensionNotSupported(t.params.viewDimension);

  const size = textureDimension === '1d' ? [4] : [4, 4, 6];
  const textureDescriptor = {
    format: 'rgba8unorm',
    dimension: textureDimension,
    size,
    usage: GPUTextureUsage.TEXTURE_BINDING
  };
  const texture = t.createTextureTracked(textureDescriptor);

  const view = { dimension: viewDimension };
  const reified = reifyTextureViewDescriptor(textureDescriptor, view);

  const success = getTextureDimensionFromView(reified.dimension) === textureDimension;
  t.expectValidationError(() => {
    texture.createView(view);
  }, !success);
});

g.test('aspect').
desc(
  `For every {format}x{aspect}, test that the view aspect must exist in the format:
  - "all" is allowed for any format
  - "depth-only" is allowed only for depth and depth-stencil formats
  - "stencil-only" is allowed only for stencil and depth-stencil formats`
).
params((u) =>
u //
.combine('format', kAllTextureFormats).
combine('aspect', kTextureAspects)
).
fn((t) => {
  const { format, aspect } = t.params;
  const { blockWidth, blockHeight } = getBlockInfoForTextureFormat(format);

  t.skipIfTextureFormatNotSupported(format);

  const texture = t.createTextureTracked({
    format,
    size: [blockWidth, blockHeight, 1],
    usage: GPUTextureUsage.TEXTURE_BINDING
  });

  const success =
  aspect === 'all' ||
  aspect === 'depth-only' && isDepthTextureFormat(format) ||
  aspect === 'stencil-only' && isStencilTextureFormat(format);
  t.expectValidationError(() => {
    texture.createView({ aspect });
  }, !success);
});

const kTextureAndViewDimensions = kUnitCaseParamsBuilder.
combine('textureDimension', kTextureDimensions).
expand('viewDimension', (p) => [
undefined,
...viewDimensionsForTextureDimension(p.textureDimension)]
);

function validateCreateViewLayersLevels(tex, view) {
  const textureLevels = tex.mipLevelCount ?? 1;
  const textureLayers = tex.dimension === '2d' ? reifyExtent3D(tex.size).depthOrArrayLayers : 1;
  const reified = reifyTextureViewDescriptor(tex, view);

  let success =
  reified.mipLevelCount > 0 &&
  reified.baseMipLevel < textureLevels &&
  reified.baseMipLevel + reified.mipLevelCount <= textureLevels &&
  reified.arrayLayerCount > 0 &&
  reified.baseArrayLayer < textureLayers &&
  reified.baseArrayLayer + reified.arrayLayerCount <= textureLayers;
  if (reified.dimension === '1d' || reified.dimension === '2d' || reified.dimension === '3d') {
    success &&= reified.arrayLayerCount === 1;
  } else if (reified.dimension === 'cube') {
    success &&= reified.arrayLayerCount === 6;
  } else if (reified.dimension === 'cube-array') {
    success &&= reified.arrayLayerCount % 6 === 0;
  }
  return success;
}

g.test('array_layers').
desc(
  `For each texture dimension {1d,2d,3d}, for each possible view dimension for that texture
    dimension (or undefined, which defaults to the texture dimension), test validation of layer
    counts:
  - 1d, 2d, and 3d must have exactly 1 layer
  - 2d-array must have 1 or more layers
  - cube must have 6 layers
  - cube-array must have a positive multiple of 6 layers
  - Defaulting of baseArrayLayer and arrayLayerCount
  - baseArrayLayer+arrayLayerCount must be within the texture`
).
params(
  kTextureAndViewDimensions.
  beginSubcases().
  expand('textureLayers', ({ textureDimension: d }) => d === '2d' ? [1, 6, 18] : [1]).
  combine('textureLevels', [1, kLevels]).
  unless((p) => p.textureDimension === '1d' && p.textureLevels !== 1).
  expand(
    'baseArrayLayer',
    ({ textureLayers: l }) => new Set([undefined, 0, 1, 5, 6, 7, l - 1, l, l + 1])
  ).
  expand('arrayLayerCount', function* ({ textureLayers: l, baseArrayLayer = 0 }) {
    yield undefined;
    for (const lastArrayLayer of new Set([0, 1, 5, 6, 7, l - 1, l, l + 1])) {
      if (baseArrayLayer <= lastArrayLayer) yield lastArrayLayer - baseArrayLayer;
    }
  })
).
fn((t) => {
  const {
    textureDimension,
    viewDimension,
    textureLayers,
    textureLevels,
    baseArrayLayer,
    arrayLayerCount
  } = t.params;

  t.skipIfTextureViewDimensionNotSupported(viewDimension);

  const kWidth = 1 << kLevels - 1; // 32
  const textureDescriptor = {
    format: 'rgba8unorm',
    dimension: textureDimension,
    size:
    textureDimension === '1d' ?
    [kWidth] :
    textureDimension === '2d' ?
    [kWidth, kWidth, textureLayers] :
    textureDimension === '3d' ?
    [kWidth, kWidth, kWidth] :
    unreachable(),
    mipLevelCount: textureLevels,
    usage: GPUTextureUsage.TEXTURE_BINDING
  };

  const viewDescriptor = { dimension: viewDimension, baseArrayLayer, arrayLayerCount };
  const success = validateCreateViewLayersLevels(textureDescriptor, viewDescriptor);

  const texture = t.createTextureTracked(textureDescriptor);
  t.expectValidationError(() => {
    texture.createView(viewDescriptor);
  }, !success);
});

g.test('mip_levels').
desc(
  `Views must have at least one level, and must be within the level of the base texture.

  - mipLevelCount=0 at various baseMipLevel values
  - Cases where baseMipLevel+mipLevelCount goes past the end of the texture
  - Cases with baseMipLevel or mipLevelCount undefined (compares against reference defaulting impl)
  `
).
params(
  kTextureAndViewDimensions.
  beginSubcases().
  combine('textureLevels', [1, kLevels - 2, kLevels]).
  unless((p) => p.textureDimension === '1d' && p.textureLevels !== 1).
  expand(
    'baseMipLevel',
    ({ textureLevels: l }) => new Set([undefined, 0, 1, 5, 6, 7, l - 1, l, l + 1])
  ).
  expand('mipLevelCount', function* ({ textureLevels: l, baseMipLevel = 0 }) {
    yield undefined;
    for (const lastMipLevel of new Set([0, 1, 5, 6, 7, l - 1, l, l + 1])) {
      if (baseMipLevel <= lastMipLevel) yield lastMipLevel - baseMipLevel;
    }
  })
).
fn((t) => {
  const { textureDimension, viewDimension, textureLevels, baseMipLevel, mipLevelCount } =
  t.params;

  t.skipIfTextureViewDimensionNotSupported(viewDimension);

  const textureDescriptor = {
    format: 'rgba8unorm',
    dimension: textureDimension,
    size:
    textureDimension === '1d' ? [32] : textureDimension === '3d' ? [32, 32, 32] : [32, 32, 18],
    mipLevelCount: textureLevels,
    usage: GPUTextureUsage.TEXTURE_BINDING
  };

  const viewDescriptor = { dimension: viewDimension, baseMipLevel, mipLevelCount };
  const success = validateCreateViewLayersLevels(textureDescriptor, viewDescriptor);

  const texture = t.createTextureTracked(textureDescriptor);
  t.debug(`${mipLevelCount} ${success}`);
  t.expectValidationError(() => {
    texture.createView(viewDescriptor);
  }, !success);
});

g.test('cube_faces_square').
desc(
  `Test that the X/Y dimensions of cube and cube array textures must be square.
  - {2d (control case), cube, cube-array}`
).
params((u) =>
u //
.combine('dimension', ['2d', 'cube', 'cube-array']).
combine('size', [
[4, 4, 6],
[5, 5, 6],
[4, 5, 6],
[4, 8, 6],
[8, 4, 6]]
)
).
fn((t) => {
  const { dimension, size } = t.params;

  t.skipIfTextureViewDimensionNotSupported(dimension);

  const texture = t.createTextureTracked({
    format: 'rgba8unorm',
    size,
    usage: GPUTextureUsage.TEXTURE_BINDING
  });

  const success = dimension === '2d' || size[0] === size[1];
  t.expectValidationError(() => {
    texture.createView({ dimension });
  }, !success);
});

g.test('texture_state').
desc(`createView should fail if the texture is invalid (but succeed if it is destroyed)`).
paramsSubcasesOnly((u) => u.combine('state', kResourceStates)).
fn((t) => {
  const { state } = t.params;
  const texture = vtu.createTextureWithState(t, state);

  t.expectValidationError(() => {
    texture.createView();
  }, state === 'invalid');
});

g.test('texture_view_usage').
desc(
  `Test texture view usage (single, combined, inherited) for every texture format and texture usage`
).
params((u) =>
u //
.combine('format', kAllTextureFormats).
combine('textureUsage', kTextureUsages).
unless(({ format, textureUsage }) => {
  return (
    (textureUsage & GPUConst.TextureUsage.RENDER_ATTACHMENT) !== 0 &&
    !isTextureFormatPossiblyUsableAsRenderAttachment(format));

}).
beginSubcases().
combine('textureViewUsage', kTextureUsages).
unless(({ textureUsage, textureViewUsage }) => {
  // TRANSIENT_ATTACHMENT is only valid when combined with RENDER_ATTACHMENT.
  return (
    textureUsage === GPUConst.TextureUsage.TRANSIENT_ATTACHMENT ||
    textureViewUsage === GPUConst.TextureUsage.TRANSIENT_ATTACHMENT);

})
).
fn((t) => {
  const { format, textureUsage, textureViewUsage } = t.params;

  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatDoesNotSupportUsage(textureUsage, format);

  const { blockWidth, blockHeight } = getBlockInfoForTextureFormat(format);

  const texture = t.createTextureTracked({
    size: [blockWidth, blockHeight, 1],
    format,
    usage: textureUsage
  });

  let success = true;

  // Texture view usage must be a subset of texture usage
  if ((~textureUsage & textureViewUsage) !== 0) success = false;

  t.expectValidationError(() => {
    texture.createView({
      usage: textureViewUsage
    });
  }, !success);
});

g.test('texture_view_usage_of_multiple_usages').
desc(
  `For a single format (rgba8unorm), check that createView:
    - allows 0 usages
    - disallows subsetting usages of TRANSIENT_ATTACHMENT textures
  `
).
params((u) =>
u.
combine('usage1', kTextureUsages).
combine('usage2', kTextureUsages).
filter((p) => p.usage1 <= p.usage2).
filter((p) => isValidTextureUsageCombination(p.usage1 | p.usage2)).
beginSubcases().
expand('viewUsage', (p) => new Set([0, p.usage1, p.usage2, p.usage1 | p.usage2]))
).
fn((t) => {
  const { usage1, usage2, viewUsage } = t.params;
  const usage = usage1 | usage2;

  // MAINTENANCE_TODO(#4509): Remove this after all implementations have TRANSIENT_ATTACHMENT.
  if ((usage & GPUConst.TextureUsage.TRANSIENT_ATTACHMENT) !== 0) {
    t.skipIfTransientAttachmentNotSupported();
  }

  let isValid = true;
  if (usage & GPUTextureUsage.TRANSIENT_ATTACHMENT) {
    isValid &&= viewUsage === usage;
  }

  const texture = t.createTextureTracked({ format: 'rgba8unorm', size: [1, 1], usage });
  t.expectGPUError(
    'validation',
    () => {
      texture.createView({ usage: viewUsage });
    },
    !isValid
  );
});

g.test('texture_view_usage_with_view_format').
desc(
  `Test that the texture view usage must be supported by the view's format. Checks for every view
    format possible, and every usage supported by the texture's format`
).
params((u) =>
u.
combine('textureFormat', kAllTextureFormats).
combine('usage', kTextureUsages).
beginSubcases().
combine('viewFormat', kAllTextureFormats).
unless(({ usage }) => {
  // TRANSIENT_ATTACHMENT is only valid when combined with RENDER_ATTACHMENT.
  return usage === GPUConst.TextureUsage.TRANSIENT_ATTACHMENT;
})
).
fn((t) => {
  const { textureFormat, viewFormat, usage } = t.params;

  t.skipIfTextureFormatNotSupported(textureFormat, viewFormat);
  t.skipIfTextureFormatDoesNotSupportUsage(usage, textureFormat);

  if (!textureFormatsAreViewCompatible(t.device.features, textureFormat, viewFormat)) {
    t.skip(`"${textureFormat}" and "${viewFormat}" are not view-compatible`);
  }

  const { blockWidth, blockHeight } = getBlockInfoForTextureFormat(textureFormat);
  const texture = t.createTextureTracked({
    size: [blockWidth, blockHeight, 1],
    format: textureFormat,
    usage,
    viewFormats: [viewFormat]
  });

  let success = true;

  // Texture view usage must be a subset of texture usage
  if (usage & GPUTextureUsage.STORAGE_BINDING) {
    if (!isTextureFormatUsableWithStorageAccessMode(t.device.features, viewFormat, 'write-only'))
    success = false;
  }
  if (usage & GPUTextureUsage.RENDER_ATTACHMENT) {
    if (
    isColorTextureFormat(viewFormat) &&
    !isTextureFormatColorRenderable(t.device.features, viewFormat))

    success = false;
  }

  // Test with explicitly setting the view usage.
  t.expectValidationError(() => {
    texture.createView({
      usage,
      format: viewFormat
    });
  }, !success);

  // Test with inheriting the view usage.
  t.expectValidationError(() => {
    texture.createView({
      format: viewFormat
    });
  }, !success);
});