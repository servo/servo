/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
render pass descriptor validation tests.

TODO: review for completeness
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { kMaxColorAttachmentsToTest, kQueryTypes } from '../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import {
  computeBytesPerSampleFromFormats,
  kDepthStencilFormats,
  kRenderableColorTextureFormats,
  kTextureFormatInfo } from
'../../../format_info.js';
import { ValidationTest } from '../validation_test.js';

class F extends ValidationTest {
  createTexture(
  options =








  {})
  {
    const {
      format = 'rgba8unorm',
      dimension = '2d',
      width = 16,
      height = 16,
      arrayLayerCount = 1,
      mipLevelCount = 1,
      sampleCount = 1,
      usage = GPUTextureUsage.RENDER_ATTACHMENT
    } = options;

    return this.device.createTexture({
      size: { width, height, depthOrArrayLayers: arrayLayerCount },
      format,
      dimension,
      mipLevelCount,
      sampleCount,
      usage
    });
  }

  getColorAttachment(
  texture,
  textureViewDescriptor)
  {
    const view = texture.createView(textureViewDescriptor);

    return {
      view,
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    };
  }

  getDepthStencilAttachment(
  texture,
  textureViewDescriptor)
  {
    const view = texture.createView(textureViewDescriptor);

    return {
      view,
      depthClearValue: 1.0,
      depthLoadOp: 'clear',
      depthStoreOp: 'store',
      stencilClearValue: 0,
      stencilLoadOp: 'clear',
      stencilStoreOp: 'store'
    };
  }

  tryRenderPass(success, descriptor) {
    const commandEncoder = this.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass(descriptor);
    renderPass.end();

    this.expectValidationError(() => {
      commandEncoder.finish();
    }, !success);
  }
}

export const g = makeTestGroup(F);
const kArrayLayerCount = 10;

g.test('attachments,one_color_attachment').
desc(`Test that a render pass works with only one color attachment.`).
fn((t) => {
  const colorTexture = t.createTexture({ format: 'rgba8unorm' });
  const descriptor = {
    colorAttachments: [t.getColorAttachment(colorTexture)]
  };

  t.tryRenderPass(true, descriptor);
});

g.test('attachments,one_depth_stencil_attachment').
desc(`Test that a render pass works with only one depthStencil attachment.`).
fn((t) => {
  const depthStencilTexture = t.createTexture({ format: 'depth24plus-stencil8' });
  const descriptor = {
    colorAttachments: [],
    depthStencilAttachment: t.getDepthStencilAttachment(depthStencilTexture)
  };

  t.tryRenderPass(true, descriptor);
});

g.test('color_attachments,empty').
desc(
  `
  Test that when colorAttachments has all values be 'undefined' or the sequence is empty, the
  depthStencilAttachment must not be 'undefined'.
  `
).
paramsSubcasesOnly((u) =>
u.
combine('unclampedColorAttachments', [
[],
[undefined],
[undefined, undefined],
new Array(8).fill(undefined),
[{ format: 'rgba8unorm' }]]
).
combine('hasDepthStencilAttachment', [false, true])
).
fn((t) => {
  const { unclampedColorAttachments, hasDepthStencilAttachment } = t.params;
  const colorAttachments = unclampedColorAttachments.slice(
    0,
    t.device.limits.maxColorAttachments
  );

  let isEmptyColorTargets = true;
  for (let i = 0; i < colorAttachments.length; i++) {
    if (colorAttachments[i] !== undefined) {
      isEmptyColorTargets = false;
      const colorTexture = t.createTexture();
      colorAttachments[i] = t.getColorAttachment(colorTexture);
    }
  }

  const _success = !isEmptyColorTargets || hasDepthStencilAttachment;
  t.tryRenderPass(_success, {
    colorAttachments,
    depthStencilAttachment: hasDepthStencilAttachment ?
    t.getDepthStencilAttachment(t.createTexture({ format: 'depth24plus-stencil8' })) :
    undefined
  });
});

g.test('color_attachments,limits,maxColorAttachments').
desc(
  `
  Test that the out of bound of color attachment indexes are handled.
    - a validation error is generated when color attachments exceed the maximum limit(8).
  `
).
paramsSimple([
{ colorAttachmentsCountVariant: { mult: 1, add: 0 }, _success: true }, // Control case
{ colorAttachmentsCountVariant: { mult: 1, add: 1 }, _success: false } // Out of bounds
]).
fn((t) => {
  const { colorAttachmentsCountVariant, _success } = t.params;
  const colorAttachmentsCount = t.makeLimitVariant(
    'maxColorAttachments',
    colorAttachmentsCountVariant
  );

  const colorAttachments = [];
  for (let i = 0; i < colorAttachmentsCount; i++) {
    const colorTexture = t.createTexture({ format: 'r8unorm' });
    colorAttachments.push(t.getColorAttachment(colorTexture));
  }

  t.tryRenderPass(_success, { colorAttachments });
});

g.test('color_attachments,limits,maxColorAttachmentBytesPerSample,aligned').
desc(
  `
    Test that the total bytes per sample of the formats of the color attachments must be no greater
    than maxColorAttachmentBytesPerSample when the components are aligned (same format).
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
beginSubcases().
combine(
  'attachmentCount',
  range(kMaxColorAttachmentsToTest, (i) => i + 1)
)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn((t) => {
  const { format, attachmentCount } = t.params;
  const info = kTextureFormatInfo[format];

  t.skipIf(
    attachmentCount > t.device.limits.maxColorAttachments,
    `attachmentCount: ${attachmentCount} > maxColorAttachments: ${t.device.limits.maxColorAttachments}`
  );

  const colorAttachments = [];
  for (let i = 0; i < attachmentCount; i++) {
    const colorTexture = t.createTexture({ format });
    colorAttachments.push(t.getColorAttachment(colorTexture));
  }
  const shouldError =
  info.colorRender === undefined ||
  computeBytesPerSampleFromFormats(range(attachmentCount, () => format)) >
  t.device.limits.maxColorAttachmentBytesPerSample;

  t.tryRenderPass(!shouldError, { colorAttachments });
});

g.test('color_attachments,limits,maxColorAttachmentBytesPerSample,unaligned').
desc(
  `
    Test that the total bytes per sample of the formats of the color attachments must be no greater
    than maxColorAttachmentBytesPerSample when the components are (potentially) unaligned.
  `
).
params((u) =>
u.combineWithParams([
// Alignment causes the first 1 byte R8Unorm to become 4 bytes. So even though
// 1+4+8+16+1 < 32, the 4 byte alignment requirement of R32Float makes the first R8Unorm
// become 4 and 4+4+8+16+1 > 32. Re-ordering this so the R8Unorm's are at the end, however
// is allowed: 4+8+16+1+1 < 32.
{
  formats: [
  'r8unorm',
  'r32float',
  'rgba8unorm',
  'rgba32float',
  'r8unorm']

},
{
  formats: [
  'r32float',
  'rgba8unorm',
  'rgba32float',
  'r8unorm',
  'r8unorm']

}]
)
).
fn((t) => {
  const { formats } = t.params;

  t.skipIf(
    formats.length > t.device.limits.maxColorAttachments,
    `numColorAttachments: ${formats.length} > maxColorAttachments: ${t.device.limits.maxColorAttachments}`
  );

  const colorAttachments = [];
  for (const format of formats) {
    const colorTexture = t.createTexture({ format });
    colorAttachments.push(t.getColorAttachment(colorTexture));
  }

  const success =
  computeBytesPerSampleFromFormats(formats) <= t.device.limits.maxColorAttachmentBytesPerSample;

  t.tryRenderPass(success, { colorAttachments });
});

g.test('color_attachments,depthSlice,definedness').
desc(
  `
  Test that depthSlice must be undefined for 2d color attachments and defined for 3d color attachments."
  - The special value '0xFFFFFFFF' is not treated as 'undefined'.
  `
).
params((u) =>
u.
combine('dimension', ['2d', '3d']).
beginSubcases().
combine('depthSlice', [undefined, 0, 0xffffffff])
).
fn((t) => {
  const { dimension, depthSlice } = t.params;
  const texture = t.createTexture({ dimension });

  const colorAttachment = t.getColorAttachment(texture);
  if (depthSlice !== undefined) {
    colorAttachment.depthSlice = depthSlice;
  }

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  const success =
  dimension === '2d' && depthSlice === undefined || dimension === '3d' && depthSlice === 0;

  t.tryRenderPass(success, descriptor);
});

g.test('color_attachments,depthSlice,bound_check').
desc(
  `
  Test that depthSlice must be less than the depthOrArrayLayers of 3d texture's subresource at mip levels.
  - Check depth bounds with 3d texture size [16, 1, 10], which has 5 mip levels with depth [10, 5, 2, 1, 1]
    for testing more mip level size computation.
  - Failed if depthSlice >= the depth of each mip level.
  `
).
params((u) =>
u.
combine('mipLevel', [0, 1, 2, 3, 4]).
beginSubcases().
expand('depthSlice', ({ mipLevel }) => {
  const depthAtMipLevel = Math.max(kArrayLayerCount >> mipLevel, 1);
  // Use Set() to exclude duplicates when the depthAtMipLevel is 1 and 2
  return [...new Set([0, 1, depthAtMipLevel - 1, depthAtMipLevel])];
})
).
fn((t) => {
  const { mipLevel, depthSlice } = t.params;

  const texture = t.createTexture({
    dimension: '3d',
    width: 16,
    height: 1,
    arrayLayerCount: kArrayLayerCount,
    mipLevelCount: mipLevel + 1
  });

  const viewDescriptor = {
    baseMipLevel: mipLevel,
    mipLevelCount: 1,
    baseArrayLayer: 0,
    arrayLayerCount: 1
  };

  const colorAttachment = t.getColorAttachment(texture, viewDescriptor);
  colorAttachment.depthSlice = depthSlice;

  const passDescriptor = {
    colorAttachments: [colorAttachment]
  };

  const success = depthSlice < Math.max(kArrayLayerCount >> mipLevel, 1);

  t.tryRenderPass(success, passDescriptor);
});

g.test('color_attachments,depthSlice,overlaps,same_miplevel').
desc(
  `
  Test that the depth slices of 3d color attachments have no overlaps for same texture in a render
  pass.
  - Succeed if the depth slices are different, or from different textures, or on different render
    passes.
  - Fail if same depth slice from same texture's same mip level is overwritten in a render pass.
  `
).
params((u) =>
u.
combine('sameDepthSlice', [true, false]).
beginSubcases().
combine('sameTexture', [true, false]).
combine('samePass', [true, false])
).
fn((t) => {
  const { sameDepthSlice, sameTexture, samePass } = t.params;
  const arrayLayerCount = 4;

  const texDescriptor = {
    dimension: '3d',
    arrayLayerCount
  };
  const texture = t.createTexture(texDescriptor);

  const colorAttachments = [];
  for (let i = 0; i < arrayLayerCount; i++) {
    const colorAttachment = t.getColorAttachment(
      sameTexture ? texture : t.createTexture(texDescriptor)
    );
    colorAttachment.depthSlice = sameDepthSlice ? 0 : i;
    colorAttachments.push(colorAttachment);
  }

  const encoder = t.createEncoder('non-pass');
  if (samePass) {
    const pass = encoder.encoder.beginRenderPass({ colorAttachments });
    pass.end();
  } else {
    for (let i = 0; i < arrayLayerCount; i++) {
      const pass = encoder.encoder.beginRenderPass({ colorAttachments: [colorAttachments[i]] });
      pass.end();
    }
  }

  const success = !sameDepthSlice || !sameTexture || !samePass;

  encoder.validateFinish(success);
});

g.test('color_attachments,depthSlice,overlaps,diff_miplevel').
desc(
  `
  Test that the same depth slice from different mip levels of a 3d texture with size [1, 1, N] can
  be set in a render pass's color attachments.
  `
).
params((u) => u.combine('sameMipLevel', [true, false])).
fn((t) => {
  const { sameMipLevel } = t.params;
  const mipLevelCount = 4;

  const texDescriptor = {
    dimension: '3d',
    width: 1,
    height: 1,
    arrayLayerCount: 1 << mipLevelCount,
    mipLevelCount
  };
  const texture = t.createTexture(texDescriptor);

  const viewDescriptor = {
    baseMipLevel: 0,
    mipLevelCount: 1,
    baseArrayLayer: 0,
    arrayLayerCount: 1
  };

  const colorAttachments = [];
  for (let i = 0; i < mipLevelCount; i++) {
    if (!sameMipLevel) {
      viewDescriptor.baseMipLevel = i;
    }
    const colorAttachment = t.getColorAttachment(texture, viewDescriptor);
    colorAttachment.depthSlice = 0;
    colorAttachments.push(colorAttachment);
  }

  const encoder = t.createEncoder('non-pass');
  const pass = encoder.encoder.beginRenderPass({ colorAttachments });
  pass.end();

  encoder.validateFinish(!sameMipLevel);
});

g.test('attachments,same_size').
desc(
  `
  Test that attachments have the same size. Otherwise, a validation error should be generated.
    - Succeed if all attachments have the same size.
    - Fail if one of the color attachments has a different size.
    - Fail if the depth stencil attachment has a different size.
  `
).
fn((t) => {
  const colorTexture1x1A = t.createTexture({ width: 1, height: 1, format: 'rgba8unorm' });
  const colorTexture1x1B = t.createTexture({ width: 1, height: 1, format: 'rgba8unorm' });
  const colorTexture2x2 = t.createTexture({ width: 2, height: 2, format: 'rgba8unorm' });
  const depthStencilTexture1x1 = t.createTexture({
    width: 1,
    height: 1,
    format: 'depth24plus-stencil8'
  });
  const depthStencilTexture2x2 = t.createTexture({
    width: 2,
    height: 2,
    format: 'depth24plus-stencil8'
  });

  {
    // Control case: all the same size (1x1)
    const descriptor = {
      colorAttachments: [
      t.getColorAttachment(colorTexture1x1A),
      t.getColorAttachment(colorTexture1x1B)],

      depthStencilAttachment: t.getDepthStencilAttachment(depthStencilTexture1x1)
    };

    t.tryRenderPass(true, descriptor);
  }
  {
    // One of the color attachments has a different size
    const descriptor = {
      colorAttachments: [
      t.getColorAttachment(colorTexture1x1A),
      t.getColorAttachment(colorTexture2x2)]

    };

    t.tryRenderPass(false, descriptor);
  }
  {
    // The depth stencil attachment has a different size
    const descriptor = {
      colorAttachments: [
      t.getColorAttachment(colorTexture1x1A),
      t.getColorAttachment(colorTexture1x1B)],

      depthStencilAttachment: t.getDepthStencilAttachment(depthStencilTexture2x2)
    };

    t.tryRenderPass(false, descriptor);
  }
});

g.test('attachments,color_depth_mismatch').
desc(`Test that attachments match whether they are used for color or depth stencil.`).
fn((t) => {
  const colorTexture = t.createTexture({ format: 'rgba8unorm' });
  const depthStencilTexture = t.createTexture({ format: 'depth24plus-stencil8' });

  {
    // Using depth-stencil for color
    const descriptor = {
      colorAttachments: [t.getColorAttachment(depthStencilTexture)]
    };

    t.tryRenderPass(false, descriptor);
  }
  {
    // Using color for depth-stencil
    const descriptor = {
      colorAttachments: [],
      depthStencilAttachment: t.getDepthStencilAttachment(colorTexture)
    };

    t.tryRenderPass(false, descriptor);
  }
});

g.test('attachments,layer_count').
desc(
  `
  Test the layer counts for color or depth stencil.
    - Fail if using 2D array texture view with arrayLayerCount > 1.
    - Succeed if using 2D array texture view that covers the first layer of the texture.
    - Succeed if using 2D array texture view that covers the last layer for depth stencil.
  `
).
paramsSimple([
{ arrayLayerCount: 5, baseArrayLayer: 0, _success: false },
{ arrayLayerCount: 1, baseArrayLayer: 0, _success: true },
{ arrayLayerCount: 1, baseArrayLayer: 9, _success: true }]
).
fn((t) => {
  const { arrayLayerCount, baseArrayLayer, _success } = t.params;

  const ARRAY_LAYER_COUNT = 10;
  const MIP_LEVEL_COUNT = 1;
  const COLOR_FORMAT = 'rgba8unorm';
  const DEPTH_STENCIL_FORMAT = 'depth24plus-stencil8';

  const colorTexture = t.createTexture({
    format: COLOR_FORMAT,
    width: 32,
    height: 32,
    mipLevelCount: MIP_LEVEL_COUNT,
    arrayLayerCount: ARRAY_LAYER_COUNT
  });
  const depthStencilTexture = t.createTexture({
    format: DEPTH_STENCIL_FORMAT,
    width: 32,
    height: 32,
    mipLevelCount: MIP_LEVEL_COUNT,
    arrayLayerCount: ARRAY_LAYER_COUNT
  });

  const baseTextureViewDescriptor = {
    dimension: '2d-array',
    baseArrayLayer,
    arrayLayerCount,
    baseMipLevel: 0,
    mipLevelCount: MIP_LEVEL_COUNT
  };

  {
    // Check 2D array texture view for color
    const textureViewDescriptor = {
      ...baseTextureViewDescriptor,
      format: COLOR_FORMAT
    };

    const descriptor = {
      colorAttachments: [t.getColorAttachment(colorTexture, textureViewDescriptor)]
    };

    t.tryRenderPass(_success, descriptor);
  }
  {
    // Check 2D array texture view for depth stencil
    const textureViewDescriptor = {
      ...baseTextureViewDescriptor,
      format: DEPTH_STENCIL_FORMAT
    };

    const descriptor = {
      colorAttachments: [],
      depthStencilAttachment: t.getDepthStencilAttachment(
        depthStencilTexture,
        textureViewDescriptor
      )
    };

    t.tryRenderPass(_success, descriptor);
  }
});

g.test('attachments,mip_level_count').
desc(
  `
  Test the mip level count for color or depth stencil.
    - Fail if using 2D texture view with mipLevelCount > 1.
    - Succeed if using 2D texture view that covers the first level of the texture.
    - Succeed if using 2D texture view that covers the last level of the texture.
  `
).
paramsSimple([
{ mipLevelCount: 2, baseMipLevel: 0, _success: false },
{ mipLevelCount: 1, baseMipLevel: 0, _success: true },
{ mipLevelCount: 1, baseMipLevel: 3, _success: true }]
).
fn((t) => {
  const { mipLevelCount, baseMipLevel, _success } = t.params;

  const ARRAY_LAYER_COUNT = 1;
  const MIP_LEVEL_COUNT = 4;
  const COLOR_FORMAT = 'rgba8unorm';
  const DEPTH_STENCIL_FORMAT = 'depth24plus-stencil8';

  const colorTexture = t.createTexture({
    format: COLOR_FORMAT,
    width: 32,
    height: 32,
    mipLevelCount: MIP_LEVEL_COUNT,
    arrayLayerCount: ARRAY_LAYER_COUNT
  });
  const depthStencilTexture = t.createTexture({
    format: DEPTH_STENCIL_FORMAT,
    width: 32,
    height: 32,
    mipLevelCount: MIP_LEVEL_COUNT,
    arrayLayerCount: ARRAY_LAYER_COUNT
  });

  const baseTextureViewDescriptor = {
    dimension: '2d',
    baseArrayLayer: 0,
    arrayLayerCount: ARRAY_LAYER_COUNT,
    baseMipLevel,
    mipLevelCount
  };

  {
    // Check 2D texture view for color
    const textureViewDescriptor = {
      ...baseTextureViewDescriptor,
      format: COLOR_FORMAT
    };

    const descriptor = {
      colorAttachments: [t.getColorAttachment(colorTexture, textureViewDescriptor)]
    };

    t.tryRenderPass(_success, descriptor);
  }
  {
    // Check 2D texture view for depth stencil
    const textureViewDescriptor = {
      ...baseTextureViewDescriptor,
      format: DEPTH_STENCIL_FORMAT
    };

    const descriptor = {
      colorAttachments: [],
      depthStencilAttachment: t.getDepthStencilAttachment(
        depthStencilTexture,
        textureViewDescriptor
      )
    };

    t.tryRenderPass(_success, descriptor);
  }
});

g.test('color_attachments,non_multisampled').
desc(
  `
  Test that setting a resolve target is invalid if the color attachments is non multisampled.
  `
).
fn((t) => {
  const colorTexture = t.createTexture({ sampleCount: 1 });
  const resolveTargetTexture = t.createTexture({ sampleCount: 1 });

  const descriptor = {
    colorAttachments: [
    {
      view: colorTexture.createView(),
      resolveTarget: resolveTargetTexture.createView(),
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }]

  };

  t.tryRenderPass(false, descriptor);
});

g.test('color_attachments,sample_count').
desc(
  `
  Test the usages of multisampled textures for color attachments.
    - Succeed if using a multisampled color attachment without setting a resolve target.
    - Fail if using multiple color attachments with different sample counts.
  `
).
fn((t) => {
  const colorTexture = t.createTexture({ sampleCount: 1 });
  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });

  {
    // It is allowed to use a multisampled color attachment without setting resolve target
    const descriptor = {
      colorAttachments: [t.getColorAttachment(multisampledColorTexture)]
    };
    t.tryRenderPass(true, descriptor);
  }
  {
    // It is not allowed to use multiple color attachments with different sample counts
    const descriptor = {
      colorAttachments: [
      t.getColorAttachment(colorTexture),
      t.getColorAttachment(multisampledColorTexture)]

    };

    t.tryRenderPass(false, descriptor);
  }
});

g.test('resolveTarget,sample_count').
desc(
  `
  Test that using multisampled resolve target is invalid for color attachments.
  `
).
fn((t) => {
  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const multisampledResolveTargetTexture = t.createTexture({ sampleCount: 4 });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = multisampledResolveTargetTexture.createView();

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  t.tryRenderPass(false, descriptor);
});

g.test('resolveTarget,array_layer_count').
desc(
  `
  Test that using a resolve target with array layer count is greater than 1 is invalid for color
  attachments.
  `
).
fn((t) => {
  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({ arrayLayerCount: 2 });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = resolveTargetTexture.createView({ dimension: '2d-array' });

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  t.tryRenderPass(false, descriptor);
});

g.test('resolveTarget,mipmap_level_count').
desc(
  `
  Test that using a resolve target with that mipmap level count is greater than 1 is invalid for
  color attachments.
  `
).
fn((t) => {
  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({ mipLevelCount: 2 });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = resolveTargetTexture.createView();

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  t.tryRenderPass(false, descriptor);
});

g.test('resolveTarget,usage').
desc(
  `
  Test that using a resolve target whose usage is not RENDER_ATTACHMENT is invalid for color
  attachments.
  `
).
paramsSimple([
{ usage: GPUConst.TextureUsage.COPY_SRC | GPUConst.TextureUsage.COPY_DST },
{ usage: GPUConst.TextureUsage.STORAGE_BINDING | GPUConst.TextureUsage.TEXTURE_BINDING },
{ usage: GPUConst.TextureUsage.STORAGE_BINDING | GPUConst.TextureUsage.STORAGE },
{ usage: GPUConst.TextureUsage.RENDER_ATTACHMENT | GPUConst.TextureUsage.TEXTURE_BINDING }]
).
fn((t) => {
  const { usage } = t.params;

  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({ usage });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = resolveTargetTexture.createView();

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  const isValid = usage & GPUConst.TextureUsage.RENDER_ATTACHMENT ? true : false;
  t.tryRenderPass(isValid, descriptor);
});

g.test('resolveTarget,error_state').
desc(`Test that a resolve target that has a error is invalid for color attachments.`).
fn((t) => {
  const ARRAY_LAYER_COUNT = 1;

  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({ arrayLayerCount: ARRAY_LAYER_COUNT });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  t.expectValidationError(() => {
    colorAttachment.resolveTarget = resolveTargetTexture.createView({
      dimension: '2d',
      format: 'rgba8unorm',
      baseArrayLayer: ARRAY_LAYER_COUNT + 1
    });
  });

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  t.tryRenderPass(false, descriptor);
});

g.test('resolveTarget,single_sample_count').
desc(
  `
  Test that a resolve target that has multi sample color attachment and a single resolve target is
  valid.
  `
).
fn((t) => {
  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({ sampleCount: 1 });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = resolveTargetTexture.createView();

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  t.tryRenderPass(true, descriptor);
});

g.test('resolveTarget,different_format').
desc(`Test that a resolve target that has a different format is invalid.`).
fn((t) => {
  const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({ format: 'bgra8unorm' });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = resolveTargetTexture.createView();

  const descriptor = {
    colorAttachments: [colorAttachment]
  };

  t.tryRenderPass(false, descriptor);
});

g.test('resolveTarget,different_size').
desc(
  `
  Test that a resolve target that has a different size with the color attachment is invalid.
  `
).
fn((t) => {
  const size = 16;
  const multisampledColorTexture = t.createTexture({ width: size, height: size, sampleCount: 4 });
  const resolveTargetTexture = t.createTexture({
    width: size * 2,
    height: size * 2,
    mipLevelCount: 2
  });

  {
    const resolveTargetTextureView = resolveTargetTexture.createView({
      baseMipLevel: 0,
      mipLevelCount: 1
    });

    const colorAttachment = t.getColorAttachment(multisampledColorTexture);
    colorAttachment.resolveTarget = resolveTargetTextureView;

    const descriptor = {
      colorAttachments: [colorAttachment]
    };

    t.tryRenderPass(false, descriptor);
  }
  {
    const resolveTargetTextureView = resolveTargetTexture.createView({ baseMipLevel: 1 });

    const colorAttachment = t.getColorAttachment(multisampledColorTexture);
    colorAttachment.resolveTarget = resolveTargetTextureView;

    const descriptor = {
      colorAttachments: [colorAttachment]
    };

    t.tryRenderPass(true, descriptor);
  }
});

g.test('depth_stencil_attachment,sample_counts_mismatch').
desc(
  `
  Test that the depth stencil attachment that has different number of samples with the color
  attachment is invalid.
  `
).
fn((t) => {
  const multisampledDepthStencilTexture = t.createTexture({
    sampleCount: 4,
    format: 'depth24plus-stencil8'
  });

  {
    // It is not allowed to use a depth stencil attachment whose sample count is different from
    // the one of the color attachment.
    const depthStencilTexture = t.createTexture({
      sampleCount: 1,
      format: 'depth24plus-stencil8'
    });
    const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
    const descriptor = {
      colorAttachments: [t.getColorAttachment(multisampledColorTexture)],
      depthStencilAttachment: t.getDepthStencilAttachment(depthStencilTexture)
    };

    t.tryRenderPass(false, descriptor);
  }
  {
    const colorTexture = t.createTexture({ sampleCount: 1 });
    const descriptor = {
      colorAttachments: [t.getColorAttachment(colorTexture)],
      depthStencilAttachment: t.getDepthStencilAttachment(multisampledDepthStencilTexture)
    };

    t.tryRenderPass(false, descriptor);
  }
  {
    // It is allowed to use a multisampled depth stencil attachment whose sample count is equal to
    // the one of the color attachment.
    const multisampledColorTexture = t.createTexture({ sampleCount: 4 });
    const descriptor = {
      colorAttachments: [t.getColorAttachment(multisampledColorTexture)],
      depthStencilAttachment: t.getDepthStencilAttachment(multisampledDepthStencilTexture)
    };

    t.tryRenderPass(true, descriptor);
  }
  {
    // It is allowed to use a multisampled depth stencil attachment with no color attachment.
    const descriptor = {
      colorAttachments: [],
      depthStencilAttachment: t.getDepthStencilAttachment(multisampledDepthStencilTexture)
    };

    t.tryRenderPass(true, descriptor);
  }
});

g.test('depth_stencil_attachment,loadOp_storeOp_match_depthReadOnly_stencilReadOnly').
desc(
  `
  Test GPURenderPassDepthStencilAttachment Usage:
    - if the format has a depth aspect:
      - if depthReadOnly is true
        - depthLoadOp and depthStoreOp must not be provided
      - else:
        - depthLoadOp and depthStoreOp must be provided
    - if the format has a stencil aspect:
      - if stencilReadOnly is true
        - stencilLoadOp and stencilStoreOp must not be provided
      - else:
        - stencilLoadOp and stencilStoreOp must be provided
  `
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
beginSubcases() // Note: It's easier to debug if you comment this line out as you can then run an individual case.
.combine('depthReadOnly', [undefined, true, false]).
combine('depthLoadOp', [undefined, 'clear', 'load']).
combine('depthStoreOp', [undefined, 'discard', 'store']).
combine('stencilReadOnly', [undefined, true, false]).
combine('stencilLoadOp', [undefined, 'clear', 'load']).
combine('stencilStoreOp', [undefined, 'discard', 'store'])
).
beforeAllSubcases((t) => {
  const info = kTextureFormatInfo[t.params.format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const {
    format,
    depthReadOnly,
    depthLoadOp,
    depthStoreOp,
    stencilReadOnly,
    stencilLoadOp,
    stencilStoreOp
  } = t.params;

  const depthAttachment = t.trackForCleanup(
    t.device.createTexture({
      format,
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    })
  );
  const depthAttachmentView = depthAttachment.createView();

  const encoder = t.device.createCommandEncoder();

  // If depthLoadOp is "clear", depthClearValue must be provided and must be between 0.0 and 1.0,
  // and it will be ignored if depthLoadOp is not "clear".
  const depthClearValue = depthLoadOp === 'clear' ? 0 : undefined;
  const renderPassDescriptor = {
    colorAttachments: [],
    depthStencilAttachment: {
      view: depthAttachmentView,
      depthLoadOp,
      depthStoreOp,
      depthReadOnly,
      stencilLoadOp,
      stencilStoreOp,
      stencilReadOnly,
      depthClearValue
    }
  };
  const pass = encoder.beginRenderPass(renderPassDescriptor);
  pass.end();

  const info = kTextureFormatInfo[format];
  const hasDepthSettings = !!depthLoadOp && !!depthStoreOp && !depthReadOnly;
  const hasStencilSettings = !!stencilLoadOp && !!stencilStoreOp && !stencilReadOnly;
  const hasDepth = info.depth;
  const hasStencil = info.stencil;

  const goodAspectSettingsPresent =
  (hasDepthSettings ? hasDepth : true) && (hasStencilSettings ? hasStencil : true);

  const hasBothDepthOps = !!depthLoadOp && !!depthStoreOp;
  const hasBothStencilOps = !!stencilLoadOp && !!stencilStoreOp;
  const hasNeitherDepthOps = !depthLoadOp && !depthStoreOp;
  const hasNeitherStencilOps = !stencilLoadOp && !stencilStoreOp;

  const goodDepthCombo = hasDepth && !depthReadOnly ? hasBothDepthOps : hasNeitherDepthOps;
  const goodStencilCombo =
  hasStencil && !stencilReadOnly ? hasBothStencilOps : hasNeitherStencilOps;

  const shouldError = !goodAspectSettingsPresent || !goodDepthCombo || !goodStencilCombo;

  t.expectValidationError(() => {
    encoder.finish();
  }, shouldError);
});

g.test('depth_stencil_attachment,depth_clear_value').
desc(
  `
  Test that depthClearValue is invalid if the value is out of the range(0.0 and 1.0) only when
  depthLoadOp is 'clear'.
  `
).
params((u) =>
u.
combine('depthLoadOp', ['load', 'clear', undefined]).
combine('depthClearValue', [undefined, -1.0, 0.0, 0.5, 1.0, 1.5])
).
fn((t) => {
  const { depthLoadOp, depthClearValue } = t.params;

  const depthStencilTexture = t.createTexture({
    format: depthLoadOp === undefined ? 'stencil8' : 'depth24plus-stencil8'
  });
  const depthStencilAttachment = t.getDepthStencilAttachment(depthStencilTexture);
  depthStencilAttachment.depthClearValue = depthClearValue;
  depthStencilAttachment.depthLoadOp = depthLoadOp;
  if (depthLoadOp === undefined) {
    depthStencilAttachment.depthStoreOp = undefined;
  }

  const descriptor = {
    colorAttachments: [t.getColorAttachment(t.createTexture())],
    depthStencilAttachment
  };

  // We can not check for out of range because NaN is not out of range.
  // So (v < 0.0 || v > 1.0) would return false when depthClearValue is undefined (NaN)
  const isDepthValueInRange = depthClearValue >= 0.0 && depthClearValue <= 1.0;
  const isInvalid = depthLoadOp === 'clear' && !isDepthValueInRange;

  t.tryRenderPass(!isInvalid, descriptor);
});

g.test('resolveTarget,format_supports_resolve').
desc(
  `
  For all formats that support 'multisample', test that they can be used as a resolveTarget
  if and only if they support 'resolve'.
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
filter((t) => kTextureFormatInfo[t.format].multisample)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];

  const multisampledColorTexture = t.createTexture({ format, sampleCount: 4 });
  const resolveTarget = t.createTexture({ format });

  const colorAttachment = t.getColorAttachment(multisampledColorTexture);
  colorAttachment.resolveTarget = resolveTarget.createView();

  t.tryRenderPass(!!info.colorRender?.resolve, {
    colorAttachments: [colorAttachment]
  });
});

g.test('timestampWrites,query_set_type').
desc(
  `
  Test that all entries of the timestampWrites must have type 'timestamp'. If all query types are
  not 'timestamp', a validation error should be generated.
  `
).
params((u) =>
u //
.combine('queryType', kQueryTypes)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase(['timestamp-query']);
}).
fn((t) => {
  const { queryType } = t.params;

  const timestampWrites = {
    querySet: t.device.createQuerySet({ type: queryType, count: 2 }),
    beginningOfPassWriteIndex: 0,
    endOfPassWriteIndex: 1
  };

  const isValid = queryType === 'timestamp';

  const colorTexture = t.createTexture();
  const descriptor = {
    colorAttachments: [t.getColorAttachment(colorTexture)],
    timestampWrites
  };

  t.tryRenderPass(isValid, descriptor);
});

g.test('timestampWrite,query_index').
desc(
  `Test that querySet.count should be greater than timestampWrite.queryIndex, and that the
         query indexes are unique.`
).
paramsSubcasesOnly((u) =>
u //
.combine('beginningOfPassWriteIndex', [undefined, 0, 1, 2, 3]).
combine('endOfPassWriteIndex', [undefined, 0, 1, 2, 3])
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase(['timestamp-query']);
}).
fn((t) => {
  const { beginningOfPassWriteIndex, endOfPassWriteIndex } = t.params;

  const querySetCount = 2;

  const timestampWrites = {
    querySet: t.device.createQuerySet({ type: 'timestamp', count: querySetCount }),
    beginningOfPassWriteIndex,
    endOfPassWriteIndex
  };

  const isValid =
  beginningOfPassWriteIndex !== endOfPassWriteIndex && (
  beginningOfPassWriteIndex === undefined || beginningOfPassWriteIndex < querySetCount) && (
  endOfPassWriteIndex === undefined || endOfPassWriteIndex < querySetCount);

  const colorTexture = t.createTexture();
  const descriptor = {
    colorAttachments: [t.getColorAttachment(colorTexture)],
    timestampWrites
  };

  t.tryRenderPass(isValid, descriptor);
});

g.test('occlusionQuerySet,query_set_type').
desc(`Test that occlusionQuerySet must have type 'occlusion'.`).
params((u) => u.combine('queryType', kQueryTypes)).
beforeAllSubcases((t) => {
  if (t.params.queryType === 'timestamp') {
    t.selectDeviceOrSkipTestCase(['timestamp-query']);
  }
}).
fn((t) => {
  const { queryType } = t.params;

  const querySet = t.device.createQuerySet({
    type: queryType,
    count: 1
  });

  const colorTexture = t.createTexture();
  const descriptor = {
    colorAttachments: [t.getColorAttachment(colorTexture)],
    occlusionQuerySet: querySet
  };

  const isValid = queryType === 'occlusion';
  t.tryRenderPass(isValid, descriptor);
});