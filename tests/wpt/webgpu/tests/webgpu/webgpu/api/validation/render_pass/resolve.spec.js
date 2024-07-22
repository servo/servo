/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for render pass resolve.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { GPUConst } from '../../../constants.js';
import { ValidationTest } from '../validation_test.js';

const kNumColorAttachments = 4;

export const g = makeTestGroup(ValidationTest);

g.test('resolve_attachment').
desc(
  `
Test various validation behaviors when a resolveTarget is provided.

- base case (valid).
- resolve source is not multisampled.
- resolve target is not single sampled.
- resolve target missing RENDER_ATTACHMENT usage.
- resolve target must have exactly one subresource:
    - base mip level {0, >0}, mip level count {1, >1}.
    - base array layer {0, >0}, array layer count {1, >1}.
- resolve target GPUTextureView is invalid
- resolve source and target have different formats.
    - rgba8unorm -> {bgra8unorm, rgba8unorm-srgb}
    - {bgra8unorm, rgba8unorm-srgb} -> rgba8unorm
    - test with other color attachments having a different format
- resolve source and target have different sizes.
`
).
paramsSimple([
// control case should be valid
{ _valid: true },
// a single sampled resolve source should cause a validation error.
{ colorAttachmentSamples: 1, _valid: false },
// a multisampled resolve target should cause a validation error.
{ resolveTargetSamples: 4, _valid: false },
// resolveTargetUsage without RENDER_ATTACHMENT usage should cause a validation error.
{ resolveTargetUsage: GPUConst.TextureUsage.COPY_SRC, _valid: false },
// non-zero resolve target base mip level should be valid.
{
  resolveTargetViewBaseMipLevel: 1,
  resolveTargetHeight: 4,
  resolveTargetWidth: 4,
  _valid: true
},
// a validation error should be created when resolveTarget is invalid.
{ resolveTargetInvalid: true, _valid: false },
// a validation error should be created when mip count > 1
{ resolveTargetViewMipCount: 2, _valid: false },
{
  resolveTargetViewBaseMipLevel: 1,
  resolveTargetViewMipCount: 2,
  resolveTargetHeight: 4,
  resolveTargetWidth: 4,
  _valid: false
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
{ resolveTargetWidth: 4, _valid: false }]
).
fn((t) => {
  const {
    colorAttachmentFormat = 'rgba8unorm',
    resolveTargetFormat = 'rgba8unorm',
    otherAttachmentFormat = 'rgba8unorm',
    colorAttachmentSamples = 4,
    resolveTargetSamples = 1,
    resolveTargetUsage = GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    resolveTargetInvalid = false,
    resolveTargetViewMipCount = 1,
    resolveTargetViewBaseMipLevel = 0,
    resolveTargetViewArrayLayerCount = 1,
    resolveTargetViewBaseArrayLayer = 0,
    colorAttachmentHeight = 2,
    colorAttachmentWidth = 2,
    resolveTargetHeight = 2,
    resolveTargetWidth = 2,
    _valid
  } = t.params;

  // Run the test in a nested loop such that the configured color attachment with resolve target
  // is tested while occupying each individual colorAttachment slot.
  for (let resolveSlot = 0; resolveSlot < kNumColorAttachments; resolveSlot++) {
    const renderPassColorAttachmentDescriptors = [];
    for (
    let colorAttachmentSlot = 0;
    colorAttachmentSlot < kNumColorAttachments;
    colorAttachmentSlot++)
    {
      // resolveSlot === colorAttachmentSlot denotes the color attachment slot that contains the
      // color attachment with resolve target.
      if (resolveSlot === colorAttachmentSlot) {
        // Create the color attachment with resolve target with the configurable parameters.
        const resolveSourceColorAttachment = t.createTextureTracked({
          format: colorAttachmentFormat,
          size: {
            width: colorAttachmentWidth,
            height: colorAttachmentHeight,
            depthOrArrayLayers: 1
          },
          sampleCount: colorAttachmentSamples,
          usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
        });

        const resolveTarget = t.createTextureTracked({
          format: resolveTargetFormat,
          size: {
            width: resolveTargetWidth,
            height: resolveTargetHeight,
            depthOrArrayLayers:
            resolveTargetViewBaseArrayLayer + resolveTargetViewArrayLayerCount
          },
          sampleCount: resolveTargetSamples,
          mipLevelCount: resolveTargetViewBaseMipLevel + resolveTargetViewMipCount,
          usage: resolveTargetUsage
        });

        renderPassColorAttachmentDescriptors.push({
          view: resolveSourceColorAttachment.createView(),
          loadOp: 'load',
          storeOp: 'discard',
          resolveTarget: resolveTargetInvalid ?
          t.getErrorTextureView() :
          resolveTarget.createView({
            dimension: resolveTargetViewArrayLayerCount === 1 ? '2d' : '2d-array',
            mipLevelCount: resolveTargetViewMipCount,
            arrayLayerCount: resolveTargetViewArrayLayerCount,
            baseMipLevel: resolveTargetViewBaseMipLevel,
            baseArrayLayer: resolveTargetViewBaseArrayLayer
          })
        });
      } else {
        // Create a basic texture to fill other color attachment slots. This texture's dimensions
        // and sample count must match the resolve source color attachment to be valid.
        const colorAttachment = t.createTextureTracked({
          format: otherAttachmentFormat,
          size: {
            width: colorAttachmentWidth,
            height: colorAttachmentHeight,
            depthOrArrayLayers: 1
          },
          sampleCount: colorAttachmentSamples,
          usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
        });

        const resolveTarget = t.createTextureTracked({
          format: otherAttachmentFormat,
          size: {
            width: colorAttachmentWidth,
            height: colorAttachmentHeight,
            depthOrArrayLayers: 1
          },
          sampleCount: 1,
          usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
        });

        renderPassColorAttachmentDescriptors.push({
          view: colorAttachment.createView(),
          loadOp: 'load',
          storeOp: 'discard',
          resolveTarget: resolveTarget.createView()
        });
      }
    }
    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: renderPassColorAttachmentDescriptors
    });
    pass.end();

    t.expectValidationError(() => {
      encoder.finish();
    }, !_valid);
  }
});