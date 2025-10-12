/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
createRenderBundleEncoder validation tests.

TODO(#3363): Make this into a MaxLimitTest and increase kMaxColorAttachments.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { getDefaultLimits } from '../../../capability_info.js';
import {
  computeBytesPerSampleFromFormats,
  getColorRenderByteCost,
  isDepthOrStencilTextureFormat,
  isTextureFormatColorRenderable,
  kAllTextureFormats,
  kDepthStencilFormats,
  kPossibleColorRenderableTextureFormats } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

// MAINTENANCE_TODO: This should be changed to kMaxColorAttachmentsToTest
// when this is made a MaxLimitTest (see above).
const kMaxColorAttachments = getDefaultLimits('core').maxColorAttachments.default;

export const g = makeTestGroup(AllFeaturesMaxLimitsGPUTest);

g.test('attachment_state,limits,maxColorAttachments').
desc(`Tests that attachment state must have <= device.limits.maxColorAttachments.`).
params((u) =>
u.beginSubcases().combine(
  'colorFormatCount',
  range(kMaxColorAttachments, (i) => i + 1)
)
).
fn((t) => {
  const { colorFormatCount } = t.params;
  const maxColorAttachments = t.device.limits.maxColorAttachments;
  t.skipIf(
    colorFormatCount > maxColorAttachments,
    `${colorFormatCount} > maxColorAttachments: ${maxColorAttachments}`
  );
  t.expectValidationError(() => {
    t.device.createRenderBundleEncoder({
      colorFormats: Array(colorFormatCount).fill('r8unorm')
    });
  }, colorFormatCount > t.device.limits.maxColorAttachments);
});

g.test('attachment_state,limits,maxColorAttachmentBytesPerSample,aligned').
desc(
  `
    Tests that the total color attachment bytes per sample <=
    device.limits.maxColorAttachmentBytesPerSample when using the same format (aligned) for multiple
    attachments.
    `
).
params((u) =>
u.
combine('format', kPossibleColorRenderableTextureFormats).
beginSubcases().
combine(
  'colorFormatCount',
  range(kMaxColorAttachments, (i) => i + 1)
)
).
fn((t) => {
  const { format, colorFormatCount } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  const maxColorAttachments = t.device.limits.maxColorAttachments;
  t.skipIf(
    colorFormatCount > maxColorAttachments,
    `${colorFormatCount} > maxColorAttachments: ${maxColorAttachments}`
  );
  const shouldError =
  !isTextureFormatColorRenderable(t.device, format) ||
  getColorRenderByteCost(format) * colorFormatCount >
  t.device.limits.maxColorAttachmentBytesPerSample;

  t.expectValidationError(() => {
    t.device.createRenderBundleEncoder({
      colorFormats: Array(colorFormatCount).fill(format)
    });
  }, shouldError);
});

g.test('attachment_state,limits,maxColorAttachmentBytesPerSample,unaligned').
desc(
  `
    Tests that the total color attachment bytes per sample <=
    device.limits.maxColorAttachmentBytesPerSample when using various sets of (potentially)
    unaligned formats.
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

  const shouldError =
  computeBytesPerSampleFromFormats(formats) > t.device.limits.maxColorAttachmentBytesPerSample;

  t.expectValidationError(() => {
    t.device.createRenderBundleEncoder({
      colorFormats: formats
    });
  }, shouldError);
});

g.test('attachment_state,empty_color_formats').
desc(`Tests that if no colorFormats are given, a depthStencilFormat must be specified.`).
params((u) =>
u.beginSubcases().combine('depthStencilFormat', [undefined, 'depth24plus-stencil8'])
).
fn((t) => {
  const { depthStencilFormat } = t.params;
  t.expectValidationError(() => {
    t.device.createRenderBundleEncoder({
      colorFormats: [],
      depthStencilFormat
    });
  }, depthStencilFormat === undefined);
});

g.test('valid_texture_formats').
desc(
  `
    Tests that createRenderBundleEncoder only accepts valid formats for its attachments.
      - colorFormats
      - depthStencilFormat
    `
).
params((u) =>
u //
.combine('format', kAllTextureFormats).
beginSubcases().
combine('attachment', ['color', 'depthStencil'])
).
fn((t) => {
  const { format, attachment } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const colorRenderable = isTextureFormatColorRenderable(t.device, format);
  const depthStencil = isDepthOrStencilTextureFormat(format);

  switch (attachment) {
    case 'color':{
        t.expectValidationError(() => {
          t.device.createRenderBundleEncoder({
            colorFormats: [format]
          });
        }, !colorRenderable);

        break;
      }
    case 'depthStencil':{
        t.expectValidationError(() => {
          t.device.createRenderBundleEncoder({
            colorFormats: [],
            depthStencilFormat: format
          });
        }, !depthStencil);

        break;
      }
  }
});

g.test('depth_stencil_readonly').
desc(
  `
      Test that allow combinations of depth-stencil format, depthReadOnly and stencilReadOnly are allowed.
    `
).
params((u) =>
u //
.combine('depthStencilFormat', kDepthStencilFormats).
beginSubcases().
combine('depthReadOnly', [false, true]).
combine('stencilReadOnly', [false, true])
).
fn((t) => {
  const { depthStencilFormat, depthReadOnly, stencilReadOnly } = t.params;
  t.skipIfTextureFormatNotSupported(depthStencilFormat);
  t.device.createRenderBundleEncoder({
    colorFormats: [],
    depthStencilFormat,
    depthReadOnly,
    stencilReadOnly
  });
});