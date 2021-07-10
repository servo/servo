/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `API Validation Tests for RenderPass Resolve.

  Test Coverage:
    - When resolveTarget is not null:
      - Test that the colorAttachment is multisampled:
        - A single sampled colorAttachment should generate an error.
      - Test that the resolveTarget is single sampled:
        - A multisampled resolveTarget should generate an error.
      - Test that the resolveTarget has usage OUTPUT_ATTACHMENT:
        - A resolveTarget without usage OUTPUT_ATTACHMENT should generate an error.
      - Test that the resolveTarget's texture view describes a single subresource:
        - A resolveTarget texture view with base mip {0, base mip > 0} and mip count of 1 should be
          valid.
          - An error should be generated when the resolve target view mip count is not 1 and base
            mip is {0, base mip > 0}.
        - A resolveTarget texture view with base array layer {0, base array layer > 0} and array
          layer count of 1 should be valid.
          - An error should be generated when the resolve target view array layer count is not 1 and
            base array layer is {0, base array layer > 0}.
      - Test that the resolveTarget's format is the same as the colorAttachment:
        - An error should be generated when the resolveTarget's format does not match the
          colorAttachment's format.
      - Test that the resolveTarget's size is the same the colorAttachment:
        - An error should be generated when the resolveTarget's height or width are not equal to
          the colorAttachment's height or width.`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';

import { ValidationTest } from './../validation_test.js';

const kNumColorAttachments = 4;

export const g = makeTestGroup(ValidationTest);

g.test('resolve_attachment')
  .params([
    // control case should be valid
    { _valid: true },
    // a single sampled resolve source should cause a validation error.
    { colorAttachmentSamples: 1, _valid: false },
    // a multisampled resolve target should cause a validation error.
    { resolveTargetSamples: 4, _valid: false },
    // resolveTargetUsage without OUTPUT_ATTACHMENT usage should cause a validation error.
    { resolveTargetUsage: GPUTextureUsage.COPY_SRC, _valid: false },
    // non-zero resolve target base mip level should be valid.
    {
      resolveTargetViewBaseMipLevel: 1,
      resolveTargetHeight: 4,
      resolveTargetWidth: 4,
      _valid: true,
    },

    // a validation error should be created when mip count > 1
    { resolveTargetViewMipCount: 2, _valid: false },
    {
      resolveTargetViewBaseMipLevel: 1,
      resolveTargetViewMipCount: 2,
      resolveTargetHeight: 4,
      resolveTargetWidth: 4,
      _valid: false,
    },

    // non-zero resolve target base array layer should be valid.
    { resolveTargetViewBaseArrayLayer: 1, _valid: true },
    // a validation error should be created when array layer count > 1
    { resolveTargetViewArrayLayerCount: 2, _valid: false },
    { resolveTargetViewBaseArrayLayer: 1, resolveTargetViewArrayLayerCount: 2, _valid: false },
    // other color attachments resolving with a different format should be valid.
    { otherAttachmentFormat: 'bgra8unorm', _valid: true },
    // mismatched colorAttachment and resolveTarget formats should cause a validation error.
    { colorAttachmentFormat: 'bgra8unorm', _valid: false },
    { colorAttachmentFormat: 'rgba8unorm-srgb', _valid: false },
    { resolveTargetFormat: 'bgra8unorm', _valid: false },
    { resolveTargetFormat: 'rgba8unorm-srgb', _valid: false },
    // mismatched colorAttachment and resolveTarget sizes should cause a validation error.
    { colorAttachmentHeight: 4, _valid: false },
    { colorAttachmentWidth: 4, _valid: false },
    { resolveTargetHeight: 4, _valid: false },
    { resolveTargetWidth: 4, _valid: false },
  ])
  .fn(async t => {
    const {
      colorAttachmentFormat = 'rgba8unorm',
      resolveTargetFormat = 'rgba8unorm',
      otherAttachmentFormat = 'rgba8unorm',
      colorAttachmentSamples = 4,
      resolveTargetSamples = 1,
      resolveTargetUsage = GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
      resolveTargetViewMipCount = 1,
      resolveTargetViewBaseMipLevel = 0,
      resolveTargetViewArrayLayerCount = 1,
      resolveTargetViewBaseArrayLayer = 0,
      colorAttachmentHeight = 2,
      colorAttachmentWidth = 2,
      resolveTargetHeight = 2,
      resolveTargetWidth = 2,
      _valid,
    } = t.params;

    // Run the test in a nested loop such that the configured color attachment with resolve target
    // is tested while occupying each individual colorAttachment slot.
    for (let resolveSlot = 0; resolveSlot < kNumColorAttachments; resolveSlot++) {
      const renderPassColorAttachmentDescriptors = [];
      for (
        let colorAttachmentSlot = 0;
        colorAttachmentSlot < kNumColorAttachments;
        colorAttachmentSlot++
      ) {
        // resolveSlot === colorAttachmentSlot denotes the color attachment slot that contains the color attachment with resolve
        // target.
        if (resolveSlot === colorAttachmentSlot) {
          // Create the color attachment with resolve target with the configurable parameters.
          const resolveSourceColorAttachment = t.device.createTexture({
            format: colorAttachmentFormat,
            size: { width: colorAttachmentWidth, height: colorAttachmentHeight, depth: 1 },
            sampleCount: colorAttachmentSamples,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
          });

          const resolveTarget = t.device.createTexture({
            format: resolveTargetFormat,
            size: {
              width: resolveTargetWidth,
              height: resolveTargetHeight,
              depth: resolveTargetViewBaseArrayLayer + resolveTargetViewArrayLayerCount,
            },

            sampleCount: resolveTargetSamples,
            mipLevelCount: resolveTargetViewBaseMipLevel + resolveTargetViewMipCount,
            usage: resolveTargetUsage,
          });

          renderPassColorAttachmentDescriptors.push({
            attachment: resolveSourceColorAttachment.createView(),
            loadValue: 'load',
            resolveTarget: resolveTarget.createView({
              dimension: resolveTargetViewArrayLayerCount === 1 ? '2d' : '2d-array',
              mipLevelCount: resolveTargetViewMipCount,
              arrayLayerCount: resolveTargetViewArrayLayerCount,
              baseMipLevel: resolveTargetViewBaseMipLevel,
              baseArrayLayer: resolveTargetViewBaseArrayLayer,
            }),
          });
        } else {
          // Create a basic texture to fill other color attachment slots. This texture's dimensions
          // and sample count must match the resolve source color attachment to be valid.
          const colorAttachment = t.device.createTexture({
            format: otherAttachmentFormat,
            size: { width: colorAttachmentWidth, height: colorAttachmentHeight, depth: 1 },
            sampleCount: colorAttachmentSamples,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
          });

          const resolveTarget = t.device.createTexture({
            format: otherAttachmentFormat,
            size: {
              width: colorAttachmentWidth,
              height: colorAttachmentHeight,
              depth: 1,
            },

            sampleCount: 1,
            usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
          });

          renderPassColorAttachmentDescriptors.push({
            attachment: colorAttachment.createView(),
            loadValue: 'load',
            resolveTarget: resolveTarget.createView(),
          });
        }
      }
      const encoder = t.device.createCommandEncoder();
      const pass = encoder.beginRenderPass({
        colorAttachments: renderPassColorAttachmentDescriptors,
      });

      pass.endPass();

      t.expectValidationError(() => {
        encoder.finish();
      }, !_valid);
    }
  });
