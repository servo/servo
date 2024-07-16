/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
This test dedicatedly tests validation of GPUFragmentState of createRenderPipeline.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, range } from '../../../../common/util/util.js';
import {
  kBlendFactors,
  kBlendOperations,
  kMaxColorAttachmentsToTest } from
'../../../capability_info.js';
import {
  kAllTextureFormats,
  kRenderableColorTextureFormats,
  kTextureFormatInfo,
  computeBytesPerSampleFromFormats,
  kColorTextureFormats } from
'../../../format_info.js';
import {
  getFragmentShaderCodeWithOutput,
  getPlainTypeInfo,
  kDefaultFragmentShaderCode,
  kDefaultVertexShaderCode } from
'../../../util/shader.js';
import { kTexelRepresentationInfo } from '../../../util/texture/texel_data.js';

import { CreateRenderPipelineValidationTest } from './common.js';

export const g = makeTestGroup(CreateRenderPipelineValidationTest);

const values = [0, 1, 0, 1];

g.test('color_target_exists').
desc(`Tests creating a complete render pipeline requires at least one color target state.`).
params((u) => u.combine('isAsync', [false, true])).
fn((t) => {
  const { isAsync } = t.params;

  const goodDescriptor = t.getDescriptor({
    targets: [{ format: 'rgba8unorm' }]
  });

  // Control case
  t.doCreateRenderPipelineTest(isAsync, true, goodDescriptor);

  // Fail because lack of color states
  const badDescriptor = t.getDescriptor({
    targets: []
  });

  t.doCreateRenderPipelineTest(isAsync, false, badDescriptor);
});

g.test('targets_format_is_color_format').
desc(
  `Tests that color target state format must be a color format, regardless of how the
    fragment shader writes to it.`
).
params((u) =>
u
// Test all non-color texture formats, plus 'rgba8unorm' as a control case.
.combine('format', kAllTextureFormats).
filter(({ format }) => {
  return format === 'rgba8unorm' || !kTextureFormatInfo[format].color;
}).
combine('isAsync', [false, true]).
beginSubcases().
combine('fragOutType', ['f32', 'u32', 'i32'])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format, fragOutType } = t.params;

  const fragmentShaderCode = getFragmentShaderCodeWithOutput([
  { values, plainType: fragOutType, componentCount: 4 }]
  );

  const success = format === 'rgba8unorm' && fragOutType === 'f32';
  t.doCreateRenderPipelineTest(isAsync, success, {
    vertex: {
      module: t.device.createShaderModule({ code: kDefaultVertexShaderCode }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({ code: fragmentShaderCode }),
      entryPoint: 'main',
      targets: [{ format }]
    },
    layout: 'auto'
  });
});

g.test('targets_format_renderable').
desc(
  `Tests that color target state format must have RENDER_ATTACHMENT capability
    (tests only color formats).`
).
params((u) =>
u //
.combine('isAsync', [false, true]).
combine('format', kColorTextureFormats)
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.skipIfTextureFormatNotSupported(t.params.format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format } = t.params;
  const info = kTextureFormatInfo[format];

  const descriptor = t.getDescriptor({ targets: [{ format }] });

  t.doCreateRenderPipelineTest(isAsync, !!info.colorRender, descriptor);
});

g.test('limits,maxColorAttachments').
desc(
  `Tests that color state targets length must not be larger than device.limits.maxColorAttachments.`
).
params((u) =>
u.combine('isAsync', [false, true]).combine('targetsLengthVariant', [
{ mult: 1, add: 0 },
{ mult: 1, add: 1 }]
)
).
fn((t) => {
  const { isAsync, targetsLengthVariant } = t.params;
  const targetsLength = t.makeLimitVariant('maxColorAttachments', targetsLengthVariant);

  const descriptor = t.getDescriptor({
    targets: range(targetsLength, (_i) => {
      return { format: 'rg8unorm', writeMask: 0 };
    }),
    fragmentShaderCode: kDefaultFragmentShaderCode,
    // add a depth stencil so that we can set writeMask to 0 for all color attachments
    depthStencil: {
      format: 'depth24plus',
      depthWriteEnabled: true,
      depthCompare: 'always'
    }
  });

  t.doCreateRenderPipelineTest(
    isAsync,
    targetsLength <= t.device.limits.maxColorAttachments,
    descriptor
  );
});

g.test('limits,maxColorAttachmentBytesPerSample,aligned').
desc(
  `
  Tests that the total color attachment bytes per sample must not be larger than
  maxColorAttachmentBytesPerSample when using the same format for multiple attachments.
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
beginSubcases().
combine(
  'attachmentCount',
  range(kMaxColorAttachmentsToTest, (i) => i + 1)
).
combine('isAsync', [false, true])
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
}).
fn((t) => {
  const { format, attachmentCount, isAsync } = t.params;
  const info = kTextureFormatInfo[format];

  t.skipIf(
    attachmentCount > t.device.limits.maxColorAttachments,
    `attachmentCount: ${attachmentCount} > maxColorAttachments: ${t.device.limits.maxColorAttachments}`
  );

  const descriptor = t.getDescriptor({
    targets: range(attachmentCount, () => {
      return { format, writeMask: 0 };
    })
  });
  const shouldError =
  info.colorRender === undefined ||
  info.colorRender.byteCost * attachmentCount >
  t.device.limits.maxColorAttachmentBytesPerSample;

  t.doCreateRenderPipelineTest(isAsync, !shouldError, descriptor);
});

g.test('limits,maxColorAttachmentBytesPerSample,unaligned').
desc(
  `
  Tests that the total color attachment bytes per sample must not be larger than
  maxColorAttachmentBytesPerSample when using various sets of (potentially) unaligned formats.
  `
).
params((u) =>
u.
combineWithParams([
// Alignment causes the first 1 byte R8Unorm to become 4 bytes. So even though
// 1+4+8+16+1 < 32, the 4 byte alignment requirement of R32Float makes the first R8Unorm
// become 4 and 4+4+8+16+1 > 32. Re-ordering this so the R8Unorm's are at the end, however
// is allowed: 4+8+16+1+1 < 32.
{
  formats: ['r8unorm', 'r32float', 'rgba8unorm', 'rgba32float', 'r8unorm']
},
{
  formats: ['r32float', 'rgba8unorm', 'rgba32float', 'r8unorm', 'r8unorm']
}]
).
beginSubcases().
combine('isAsync', [false, true])
).
fn((t) => {
  const { formats, isAsync } = t.params;

  t.skipIf(
    formats.length > t.device.limits.maxColorAttachments,
    `numColorAttachments: ${formats.length} > maxColorAttachments: ${t.device.limits.maxColorAttachments}`
  );

  const success =
  computeBytesPerSampleFromFormats(formats) <= t.device.limits.maxColorAttachmentBytesPerSample;

  const descriptor = t.getDescriptor({
    targets: formats.map((f) => {
      return { format: f, writeMask: 0 };
    })
  });

  t.doCreateRenderPipelineTest(isAsync, success, descriptor);
});

g.test('targets_format_filterable').
desc(
  `
  Tests that color target state format must be filterable if blend is not undefined.
  `
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', kRenderableColorTextureFormats).
beginSubcases().
combine('hasBlend', [false, true])
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.skipIfTextureFormatNotSupported(format);
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { isAsync, format, hasBlend } = t.params;
  const info = kTextureFormatInfo[format];

  const descriptor = t.getDescriptor({
    targets: [
    {
      format,
      blend: hasBlend ? { color: {}, alpha: {} } : undefined
    }]

  });

  const supportsBlend = info.colorRender?.blend;
  assert(supportsBlend !== undefined);
  t.doCreateRenderPipelineTest(isAsync, !hasBlend || supportsBlend, descriptor);
});

g.test('targets_blend').
desc(
  `
  For the blend components on either GPUBlendState.color or GPUBlendState.alpha:
  - Tests if the combination of 'srcFactor', 'dstFactor' and 'operation' is valid (if the blend
    operation is "min" or "max", srcFactor and dstFactor must be "one").
  `
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('component', ['color', 'alpha']).
beginSubcases().
combine('srcFactor', kBlendFactors).
combine('dstFactor', kBlendFactors).
combine('operation', kBlendOperations)
).
fn((t) => {
  const { isAsync, component, srcFactor, dstFactor, operation } = t.params;

  const defaultBlendComponent = {
    srcFactor: 'src-alpha',
    dstFactor: 'dst-alpha',
    operation: 'add'
  };
  const blendComponentToTest = {
    srcFactor,
    dstFactor,
    operation
  };
  const format = 'rgba8unorm';

  const descriptor = t.getDescriptor({
    targets: [
    {
      format,
      blend: {
        color: component === 'color' ? blendComponentToTest : defaultBlendComponent,
        alpha: component === 'alpha' ? blendComponentToTest : defaultBlendComponent
      }
    }]

  });

  if (operation === 'min' || operation === 'max') {
    const _success = srcFactor === 'one' && dstFactor === 'one';
    t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
  } else {
    t.doCreateRenderPipelineTest(isAsync, true, descriptor);
  }
});

g.test('targets_write_mask').
desc(`Tests that color target state write mask must be < 16.`).
params((u) => u.combine('isAsync', [false, true]).combine('writeMask', [0, 0xf, 0x10, 0x80000001])).
fn((t) => {
  const { isAsync, writeMask } = t.params;

  const descriptor = t.getDescriptor({
    targets: [
    {
      format: 'rgba8unorm',
      writeMask
    }]

  });

  t.doCreateRenderPipelineTest(isAsync, writeMask < 16, descriptor);
});

g.test('pipeline_output_targets').
desc(
  `Pipeline fragment output types must be compatible with target color state format
  - The scalar type (f32, i32, or u32) must match the sample type of the format.
  - The componentCount of the fragment output (e.g. f32, vec2, vec3, vec4) must not have fewer
    channels than that of the color attachment texture formats. Extra components are allowed and are discarded.

  Otherwise, color state write mask must be 0.`
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', [undefined, ...kRenderableColorTextureFormats]).
beginSubcases().
combine('shaderOutput', [
undefined,
...u.combine('scalar', ['f32', 'u32', 'i32']).combine('count', [1, 2, 3, 4])]
)
// We only care about testing writeMask if there is an attachment but no shader output.
.expand('writeMask', (p) =>
p.format !== undefined && p.shaderOutput !== undefined ? [0, 0x1, 0x2, 0x4, 0x8] : [0xf]
)
).
beforeAllSubcases((t) => {
  t.selectDeviceForTextureFormatOrSkipTestCase(t.params.format);
}).
fn((t) => {
  const { isAsync, format, writeMask, shaderOutput } = t.params;

  const descriptor = t.getDescriptor({
    targets: format ? [{ format, writeMask }] : [],
    // To have a dummy depthStencil attachment to avoid having no attachment at all which is invalid
    depthStencil: { format: 'depth24plus', depthWriteEnabled: false, depthCompare: 'always' },
    fragmentShaderCode: getFragmentShaderCodeWithOutput(
      shaderOutput ?
      [{ values, plainType: shaderOutput.scalar, componentCount: shaderOutput.count }] :
      []
    )
  });

  let success = true;
  if (format) {
    // There is a color target
    if (shaderOutput) {
      // The shader outputs to the color target
      const info = kTextureFormatInfo[format];
      success =
      shaderOutput.scalar === getPlainTypeInfo(info.color.type) &&
      shaderOutput.count >= kTexelRepresentationInfo[format].componentOrder.length;
    } else {
      // The shader does not output to the color target
      success = writeMask === 0;
    }
  }

  t.doCreateRenderPipelineTest(isAsync, success, descriptor);
});

g.test('pipeline_output_targets,blend').
desc(
  `On top of requirements from pipeline_output_targets, when blending is enabled and alpha channel is read indicated by any blend factor, an extra requirement is added:
  - fragment output must be vec4.
  `
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', ['r8unorm', 'rg8unorm', 'rgba8unorm', 'bgra8unorm']).
combine('componentCount', [1, 2, 3, 4]).
beginSubcases()
// The default srcFactor and dstFactor are 'one' and 'zero'. Override just one at a time.
.combineWithParams([
...u.combine('colorSrcFactor', kBlendFactors),
...u.combine('colorDstFactor', kBlendFactors),
...u.combine('alphaSrcFactor', kBlendFactors),
...u.combine('alphaDstFactor', kBlendFactors)]
)
).
beforeAllSubcases((t) => {
  const { format } = t.params;
  const info = kTextureFormatInfo[format];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const sampleType = 'float';
  const {
    isAsync,
    format,
    componentCount,
    colorSrcFactor,
    colorDstFactor,
    alphaSrcFactor,
    alphaDstFactor
  } = t.params;
  const info = kTextureFormatInfo[format];

  const descriptor = t.getDescriptor({
    targets: [
    {
      format,
      blend: {
        color: { srcFactor: colorSrcFactor, dstFactor: colorDstFactor },
        alpha: { srcFactor: alphaSrcFactor, dstFactor: alphaDstFactor }
      }
    }],

    fragmentShaderCode: getFragmentShaderCodeWithOutput([
    { values, plainType: getPlainTypeInfo(sampleType), componentCount }]
    )
  });

  const colorBlendReadsSrcAlpha =
  colorSrcFactor?.includes('src-alpha') || colorDstFactor?.includes('src-alpha');
  const meetsExtraBlendingRequirement = !colorBlendReadsSrcAlpha || componentCount === 4;
  const _success =
  info.color.type === sampleType &&
  componentCount >= kTexelRepresentationInfo[format].componentOrder.length &&
  meetsExtraBlendingRequirement;
  t.doCreateRenderPipelineTest(isAsync, _success, descriptor);
});