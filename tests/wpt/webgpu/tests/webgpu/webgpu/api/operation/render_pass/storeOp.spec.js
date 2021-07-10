/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `API Operation Tests for RenderPass StoreOp.

  Test Coverage:

  - Tests that color and depth-stencil store operations {'clear', 'store'} work correctly for a
    render pass with both a color attachment and depth-stencil attachment.
      TODO: use depth24plus-stencil8

  - Tests that store operations {'clear', 'store'} work correctly for a render pass with multiple
    color attachments.
      TODO: test with more interesting loadOp values

  - Tests that store operations {'clear', 'store'} work correctly for a render pass with a color
    attachment for:
      - All renderable color formats
      - mip level set to {'0', mip > '0'}
      - array layer set to {'0', layer > '1'} for 2D textures
      TODO: depth slice set to {'0', slice > '0'} for 3D textures

  - Tests that store operations {'clear', 'store'} work correctly for a render pass with a
    depth-stencil attachment for:
      - All renderable depth-stencil formats
      - mip level set to {'0', mip > '0'}
      - array layer set to {'0', layer > '1'} for 2D textures
      TODO: test depth24plus and depth24plus-stencil8 formats
      TODO: test that depth and stencil aspects are set seperately
      TODO: depth slice set to {'0', slice > '0'} for 3D textures
      TODO: test with more interesting loadOp values`;
import { params, poptions } from '../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  kEncodableTextureFormatInfo,
  kEncodableTextureFormats,
  kSizedDepthStencilFormats,
} from '../../../capability_info.js';
import { GPUTest } from '../../../gpu_test.js';

// Test with a zero and non-zero mip.
const kMipLevel = [0, 1];
const kMipLevelCount = 2;

// Test with different numbers of color attachments.

const kNumColorAttachments = [1, 2, 3, 4];

// Test with a zero and non-zero array layer.
const kArrayLayers = [0, 1];

const kStoreOps = ['clear', 'store'];

const kHeight = 2;
const kWidth = 2;

export const g = makeTestGroup(GPUTest);

// Tests a render pass with both a color and depth stencil attachment to ensure store operations are
// set independently.
g.test('render_pass_store_op,color_attachment_with_depth_stencil_attachment')
  .params(
    params()
      .combine(poptions('colorStoreOperation', kStoreOps))
      .combine(poptions('depthStencilStoreOperation', kStoreOps))
  )
  .fn(t => {
    // Create a basic color attachment.
    const kColorFormat = 'rgba8unorm';
    const colorAttachment = t.device.createTexture({
      format: kColorFormat,
      size: { width: kWidth, height: kHeight, depth: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    const colorAttachmentView = colorAttachment.createView();

    // Create a basic depth/stencil attachment.
    const kDepthStencilFormat = 'depth32float';
    const depthStencilAttachment = t.device.createTexture({
      format: kDepthStencilFormat,
      size: { width: kWidth, height: kHeight, depth: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    // Color load operation will clear to {1.0, 1.0, 1.0, 1.0}.
    // Depth & stencil load operations will clear to 1.0.
    // Store operations are determined by test the params.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: colorAttachmentView,
          loadValue: { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
          storeOp: t.params.colorStoreOperation,
        },
      ],

      depthStencilAttachment: {
        attachment: depthStencilAttachment.createView(),
        depthLoadValue: 1.0,
        depthStoreOp: t.params.depthStencilStoreOperation,
        stencilLoadValue: 1.0,
        stencilStoreOp: t.params.depthStencilStoreOperation,
      },
    });

    pass.endPass();

    t.device.defaultQueue.submit([encoder.finish()]);

    // Check that the correct store operation occurred.
    let expectedColorValue = {};
    if (t.params.colorStoreOperation === 'clear') {
      // If colorStoreOp was clear, the texture should now contain {0.0, 0.0, 0.0, 0.0}.
      expectedColorValue = { R: 0.0, G: 0.0, B: 0.0, A: 0.0 };
    } else if (t.params.colorStoreOperation === 'store') {
      // If colorStoreOP was store, the texture should still contain {1.0, 1.0, 1.0, 1.0}.
      expectedColorValue = { R: 1.0, G: 1.0, B: 1.0, A: 1.0 };
    }
    t.expectSingleColor(colorAttachment, kColorFormat, {
      size: [kHeight, kWidth, 1],
      exp: expectedColorValue,
    });

    // Check that the correct store operation occurred.
    let expectedDepthValue = {};
    if (t.params.depthStencilStoreOperation === 'clear') {
      // If depthStencilStoreOperation was clear, the texture's depth component should be 0.0, and
      // the stencil component should be 0.0.
      expectedDepthValue = { Depth: 0.0 };
    } else if (t.params.depthStencilStoreOperation === 'store') {
      // If depthStencilStoreOperation was store, the texture's depth component should be 1.0, and
      // the stencil component should be 1.0.
      expectedDepthValue = { Depth: 1.0 };
    }
    t.expectSingleColor(depthStencilAttachment, kDepthStencilFormat, {
      size: [kHeight, kWidth, 1],
      exp: expectedDepthValue,
    });
  });

// Tests that render pass color attachment store operations work correctly for all renderable color
// formats, mip levels and array layers.
g.test('render_pass_store_op,color_attachment_only')
  .params(
    params()
      .combine(poptions('colorFormat', kEncodableTextureFormats))
      // Filter out any non-renderable formats
      .filter(({ colorFormat }) => kEncodableTextureFormatInfo[colorFormat].renderable)
      .combine(poptions('storeOperation', kStoreOps))
      .combine(poptions('mipLevel', kMipLevel))
      .combine(poptions('arrayLayer', kArrayLayers))
  )
  .fn(t => {
    const colorAttachment = t.device.createTexture({
      format: t.params.colorFormat,
      size: { width: kWidth, height: kHeight, depth: t.params.arrayLayer + 1 },
      mipLevelCount: kMipLevelCount,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    const colorViewDesc = {
      baseArrayLayer: t.params.arrayLayer,
      baseMipLevel: t.params.mipLevel,
      mipLevelCount: 1,
      arrayLayerCount: 1,
    };

    const colorAttachmentView = colorAttachment.createView(colorViewDesc);

    // Color load operation will clear to {1.0, 0.0, 0.0, 1.0}.
    // Color store operation is determined by the test params.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: colorAttachmentView,
          loadValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
          storeOp: t.params.storeOperation,
        },
      ],
    });

    pass.endPass();
    t.device.defaultQueue.submit([encoder.finish()]);

    // Check that the correct store operation occurred.
    let expectedValue = {};
    if (t.params.storeOperation === 'clear') {
      // If colorStoreOp was clear, the texture should now contain {0.0, 0.0, 0.0, 0.0}.
      expectedValue = { R: 0.0, G: 0.0, B: 0.0, A: 0.0 };
    } else if (t.params.storeOperation === 'store') {
      // If colorStoreOP was store, the texture should still contain {1.0, 0.0, 0.0, 1.0}.
      expectedValue = { R: 1.0, G: 0.0, B: 0.0, A: 1.0 };
    }

    t.expectSingleColor(colorAttachment, t.params.colorFormat, {
      size: [kHeight, kWidth, 1],
      slice: t.params.arrayLayer,
      exp: expectedValue,
      layout: { mipLevel: t.params.mipLevel },
    });
  });

// Test with multiple color attachments to ensure each attachment's storeOp is set independently.
g.test('render_pass_store_op,multiple_color_attachments')
  .params(
    params()
      .combine(poptions('colorAttachments', kNumColorAttachments))
      .combine(poptions('storeOperation1', kStoreOps))
      .combine(poptions('storeOperation2', kStoreOps))
  )
  .fn(t => {
    const kColorFormat = 'rgba8unorm';
    const colorAttachments = [];

    for (let i = 0; i < t.params.colorAttachments; i++) {
      colorAttachments.push(
        t.device.createTexture({
          format: kColorFormat,
          size: { width: kWidth, height: kHeight, depth: 1 },
          usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
        })
      );
    }

    // Color load operation will clear to {1.0, 1.0, 1.0, 1.0}
    // Color store operation is determined by test params. Use storeOperation1 for even numbered
    // attachments and storeOperation2 for odd numbered attachments.
    const renderPassColorAttachmentDescriptors = [];
    for (let i = 0; i < t.params.colorAttachments; i++) {
      renderPassColorAttachmentDescriptors.push({
        attachment: colorAttachments[i].createView(),
        loadValue: { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        storeOp: i % 2 === 0 ? t.params.storeOperation1 : t.params.storeOperation2,
      });
    }

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: renderPassColorAttachmentDescriptors,
    });

    pass.endPass();
    t.device.defaultQueue.submit([encoder.finish()]);

    // Check that the correct store operation occurred.
    let expectedValue = {};
    for (let i = 0; i < t.params.colorAttachments; i++) {
      if (renderPassColorAttachmentDescriptors[i].storeOp === 'clear') {
        // If colorStoreOp was clear, the texture should now contain {0.0, 0.0, 0.0, 0.0}.
        expectedValue = { R: 0.0, G: 0.0, B: 0.0, A: 0.0 };
      } else if (renderPassColorAttachmentDescriptors[i].storeOp === 'store') {
        // If colorStoreOP was store, the texture should still contain {1.0, 1.0, 1.0, 1.0}.
        expectedValue = { R: 1.0, G: 1.0, B: 1.0, A: 1.0 };
      }
      t.expectSingleColor(colorAttachments[i], kColorFormat, {
        size: [kHeight, kWidth, 1],
        exp: expectedValue,
      });
    }
  });

// Tests that render pass depth stencil store operations work correctly for all renderable color
// formats, mip levels and array layers.
g.test('render_pass_store_op,depth_stencil_attachment_only')
  .params(
    params()
      // TODO: Also test unsized depth/stencil formats
      .combine(poptions('depthStencilFormat', kSizedDepthStencilFormats))
      .combine(poptions('storeOperation', kStoreOps))
      .combine(poptions('mipLevel', kMipLevel))
      .combine(poptions('arrayLayer', kArrayLayers))
  )
  .fn(t => {
    const depthStencilAttachment = t.device.createTexture({
      format: t.params.depthStencilFormat,
      size: { width: kWidth, height: kHeight, depth: t.params.arrayLayer + 1 },
      mipLevelCount: kMipLevelCount,
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    const depthStencilViewDesc = {
      baseArrayLayer: t.params.arrayLayer,
      baseMipLevel: t.params.mipLevel,
      mipLevelCount: 1,
      arrayLayerCount: 1,
    };

    const depthStencilAttachmentView = depthStencilAttachment.createView(depthStencilViewDesc);

    // Depth-stencil load operation will clear to depth = 1.0, stencil = 1.0.
    // Depth-stencil store operate is determined by test params.
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [],
      depthStencilAttachment: {
        attachment: depthStencilAttachmentView,
        depthLoadValue: 1.0,
        depthStoreOp: t.params.storeOperation,
        stencilLoadValue: 1.0,
        stencilStoreOp: t.params.storeOperation,
      },
    });

    pass.endPass();
    t.device.defaultQueue.submit([encoder.finish()]);

    let expectedValue = {};
    if (t.params.storeOperation === 'clear') {
      // If depthStencilStoreOperation was clear, the texture's depth component should be 0.0,
      expectedValue = { Depth: 0.0 };
    } else if (t.params.storeOperation === 'store') {
      // If depthStencilStoreOperation was store, the texture's depth component should be 1.0,
      expectedValue = { Depth: 1.0 };
    }

    t.expectSingleColor(depthStencilAttachment, t.params.depthStencilFormat, {
      size: [kHeight, kWidth, 1],
      slice: t.params.arrayLayer,
      exp: expectedValue,
      layout: { mipLevel: t.params.mipLevel },
    });
  });
