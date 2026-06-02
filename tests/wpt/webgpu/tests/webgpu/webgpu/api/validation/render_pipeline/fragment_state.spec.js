/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
This test dedicatedly tests validation of GPUFragmentState of createRenderPipeline.

TODO(#3363): Make this into a MaxLimitTest and increase kMaxColorAttachments.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, range } from '../../../../common/util/util.js';
import {
  getDefaultLimits,
  IsDualSourceBlendingFactor as isDualSourceBlendingFactor,
  kBlendFactors,
  kBlendOperations } from
'../../../capability_info.js';
import { GPUConst } from '../../../constants.js';
import {
  kAllTextureFormats,
  computeBytesPerSampleFromFormats,
  kColorTextureFormats,
  isTextureFormatColorRenderable,
  isTextureFormatBlendable,
  getTextureFormatColorType,
  isColorTextureFormat,
  kPossibleColorRenderableTextureFormats,
  getColorRenderByteCost } from
'../../../format_info.js';
import {
  getFragmentShaderCodeWithOutput,
  getPlainTypeInfo,
  kDefaultFragmentShaderCode,
  kDefaultVertexShaderCode } from
'../../../util/shader.js';
import { kTexelRepresentationInfo } from '../../../util/texture/texel_data.js';
import * as vtu from '../validation_test_utils.js';

import { CreateRenderPipelineValidationTest } from './common.js';

// MAINTENANCE_TODO: This should be changed to kMaxColorAttachmentsToTest
// when this is made a MaxLimitTest (see above).
const kMaxColorAttachments = getDefaultLimits('core').maxColorAttachments.default;

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
  vtu.doCreateRenderPipelineTest(t, isAsync, true, goodDescriptor);

  // Fail because lack of color states
  const badDescriptor = t.getDescriptor({
    targets: []
  });

  vtu.doCreateRenderPipelineTest(t, isAsync, false, badDescriptor);
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
  return format === 'rgba8unorm' || !isColorTextureFormat(format);
}).
combine('isAsync', [false, true]).
beginSubcases().
combine('fragOutType', ['f32', 'u32', 'i32'])
).
fn((t) => {
  const { isAsync, format, fragOutType } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const fragmentShaderCode = getFragmentShaderCodeWithOutput([
  { values, plainType: fragOutType, componentCount: 4 }]
  );

  const success = format === 'rgba8unorm' && fragOutType === 'f32';
  vtu.doCreateRenderPipelineTest(t, isAsync, success, {
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
fn((t) => {
  const { isAsync, format } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const descriptor = t.getDescriptor({ targets: [{ format }] });

  vtu.doCreateRenderPipelineTest(
    t,
    isAsync,
    isTextureFormatColorRenderable(t.device, format),
    descriptor
  );
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

  vtu.doCreateRenderPipelineTest(
    t,
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
combine('format', kPossibleColorRenderableTextureFormats).
beginSubcases().
combine(
  'attachmentCount',
  range(kMaxColorAttachments, (i) => i + 1)
).
combine('isAsync', [false, true])
).
fn((t) => {
  const { format, attachmentCount, isAsync } = t.params;
  t.skipIfTextureFormatNotSupported(format);

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
  !isTextureFormatColorRenderable(t.device, format) ||
  getColorRenderByteCost(format) * attachmentCount >
  t.device.limits.maxColorAttachmentBytesPerSample;

  vtu.doCreateRenderPipelineTest(t, isAsync, !shouldError, descriptor);
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

  vtu.doCreateRenderPipelineTest(t, isAsync, success, descriptor);
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
combine('format', kPossibleColorRenderableTextureFormats).
beginSubcases().
combine('hasBlend', [false, true])
).
fn((t) => {
  const { isAsync, format, hasBlend } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatNotUsableAsRenderAttachment(format);

  const descriptor = t.getDescriptor({
    targets: [
    {
      format,
      blend: hasBlend ? { color: {}, alpha: {} } : undefined
    }]

  });

  const supportsBlend = isTextureFormatBlendable(t.device, format);
  vtu.doCreateRenderPipelineTest(t, isAsync, !hasBlend || supportsBlend, descriptor);
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
combine('srcFactor', kBlendFactors).
combine('dstFactor', kBlendFactors).
beginSubcases().
combine('operation', kBlendOperations)
).
fn((t) => {
  const { isAsync, component, srcFactor, dstFactor, operation } = t.params;
  if (isDualSourceBlendingFactor(srcFactor) || isDualSourceBlendingFactor(dstFactor)) {
    t.skipIfDeviceDoesNotHaveFeature('dual-source-blending');
  }

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
  const useDualSourceBlending =
  isDualSourceBlendingFactor(srcFactor) || isDualSourceBlendingFactor(dstFactor);
  const fragmentShaderCode = getFragmentShaderCodeWithOutput(
    [{ values, plainType: 'f32', componentCount: 4 }],
    null,
    useDualSourceBlending
  );

  const descriptor = t.getDescriptor({
    targets: [
    {
      format,
      blend: {
        color: component === 'color' ? blendComponentToTest : defaultBlendComponent,
        alpha: component === 'alpha' ? blendComponentToTest : defaultBlendComponent
      }
    }],

    fragmentShaderCode
  });

  if (operation === 'min' || operation === 'max') {
    const _success = srcFactor === 'one' && dstFactor === 'one';
    vtu.doCreateRenderPipelineTest(t, isAsync, _success, descriptor);
  } else {
    vtu.doCreateRenderPipelineTest(t, isAsync, true, descriptor);
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

  vtu.doCreateRenderPipelineTest(t, isAsync, writeMask < 16, descriptor);
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
combine('format', [undefined, ...kPossibleColorRenderableTextureFormats]).
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
fn((t) => {
  const { isAsync, format, writeMask, shaderOutput } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  t.skipIfTextureFormatNotUsableAsRenderAttachment(format);

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
      success =
      shaderOutput.scalar === getPlainTypeInfo(getTextureFormatColorType(format)) &&
      shaderOutput.count >= kTexelRepresentationInfo[format].componentOrder.length;
    } else {
      // The shader does not output to the color target
      success = writeMask === 0;
    }
  }

  vtu.doCreateRenderPipelineTest(t, isAsync, success, descriptor);
});

g.test('pipeline_output_targets,blend').
desc(
  `On top of requirements from pipeline_output_targets, when blending is enabled and alpha channel
    is read indicated by any color blend factor, an extra requirement is added:
      - fragment output must be vec4.
  `
).
params((u) =>
u.
combine('isAsync', [false, true]).
combine('format', ['r8unorm', 'rg8unorm', 'rgba8unorm', 'bgra8unorm']).
combine('componentCount', [1, 2, 3, 4])
// The default srcFactor and dstFactor are 'one' and 'zero'. Override just one at a time.
.combineWithParams([
...u.combine('colorSrcFactor', kBlendFactors),
...u.combine('colorDstFactor', kBlendFactors),
...u.combine('alphaSrcFactor', kBlendFactors),
...u.combine('alphaDstFactor', kBlendFactors)]
)
).
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
  t.skipIfTextureFormatNotSupported(format);

  const useDualSourceBlending =
  isDualSourceBlendingFactor(colorSrcFactor) ||
  isDualSourceBlendingFactor(colorDstFactor) ||
  isDualSourceBlendingFactor(alphaSrcFactor) ||
  isDualSourceBlendingFactor(alphaDstFactor);
  if (useDualSourceBlending) {
    t.skipIfDeviceDoesNotHaveFeature('dual-source-blending');
  }

  const descriptor = t.getDescriptor({
    targets: [
    {
      format,
      blend: {
        color: { srcFactor: colorSrcFactor, dstFactor: colorDstFactor },
        alpha: { srcFactor: alphaSrcFactor, dstFactor: alphaDstFactor }
      }
    }],

    fragmentShaderCode: getFragmentShaderCodeWithOutput(
      [{ values, plainType: getPlainTypeInfo(sampleType), componentCount }],
      null,
      useDualSourceBlending
    )
  });

  const colorBlendReadsSrcAlpha =
  colorSrcFactor?.includes('src-alpha') ||
  colorDstFactor?.includes('src-alpha') ||
  colorSrcFactor?.includes('src1-alpha') ||
  colorDstFactor?.includes('src1-alpha');
  const meetsExtraBlendingRequirement = !colorBlendReadsSrcAlpha || componentCount === 4;
  const _success =
  getTextureFormatColorType(format) === sampleType &&
  componentCount >= kTexelRepresentationInfo[format].componentOrder.length &&
  meetsExtraBlendingRequirement;
  vtu.doCreateRenderPipelineTest(t, isAsync, _success, descriptor);
});

const kDualSourceBlendingFactors = [
'src1',
'one-minus-src1',
'src1-alpha',
'one-minus-src1-alpha'];


g.test('dual_source_blending,color_target_count').
desc(
  `Test that when the blend factor of color attachment 0 uses src1 (the second input of the
   corresponding blending unit), there must be exactly one color target.
`
).
params((u) =>
u.
combine('blendFactor', kDualSourceBlendingFactors).
combine('colorTargetsCount', [1, 2]).
combine('maskOutNonZeroIndexColorTargets', [true, false]).
beginSubcases().
combine('component', ['color', 'alpha'])
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('dual-source-blending');
  const { blendFactor, colorTargetsCount, maskOutNonZeroIndexColorTargets, component } = t.params;

  const defaultBlendComponent = {
    srcFactor: 'src-alpha',
    dstFactor: 'dst-alpha',
    operation: 'add'
  };
  const testBlendComponent = {
    srcFactor: blendFactor,
    dstFactor: blendFactor,
    operation: 'add'
  };

  assert(colorTargetsCount >= 1);
  const colorTargetStates = new Array(colorTargetsCount);
  colorTargetStates[0] = {
    format: 'rgba8unorm',
    blend: {
      color: component === 'color' ? testBlendComponent : defaultBlendComponent,
      alpha: component === 'alpha' ? testBlendComponent : defaultBlendComponent
    }
  };

  for (let i = 1; i < colorTargetsCount; ++i) {
    colorTargetStates[i] = {
      format: 'rgba8unorm',
      blend: {
        color: defaultBlendComponent,
        alpha: defaultBlendComponent
      },
      writeMask: maskOutNonZeroIndexColorTargets ? 0 : GPUConst.ColorWrite.ALL
    };
  }

  const descriptor = t.getDescriptor({
    targets: colorTargetStates,
    fragmentShaderCode: getFragmentShaderCodeWithOutput(
      [{ values, plainType: 'f32', componentCount: 4 }],
      null,
      true
    )
  });

  const isAsync = false;
  const _success = colorTargetsCount === 1;
  vtu.doCreateRenderPipelineTest(t, isAsync, _success, descriptor);
});

g.test('dual_source_blending,use_blend_src').
desc(
  `Test that when the blend factor of color attachment 0 uses src1, dual source blending must be
    used in the fragment shader, whether the corresponding color write mask is 0 or not. In
    contrast, when dual source blending is used in the fragment shader, we don't require blend
    factor must use src1 (the second input of the corresponding blending unit).
`
).
params((u) =>
u.
combine('blendFactor', kBlendFactors).
combine('useBlendSrc1', [true, false]).
combine('writeMask', [0, GPUConst.ColorWrite.ALL]).
beginSubcases().
combine('component', ['color', 'alpha'])
).
fn((t) => {
  t.skipIfDeviceDoesNotHaveFeature('dual-source-blending');
  const { blendFactor, useBlendSrc1, writeMask, component } = t.params;

  const defaultBlendComponent = {
    srcFactor: 'src-alpha',
    dstFactor: 'dst-alpha',
    operation: 'add'
  };
  const testBlendComponent = {
    srcFactor: blendFactor,
    dstFactor: blendFactor,
    operation: 'add'
  };

  const descriptor = t.getDescriptor({
    targets: [
    {
      format: 'rgba8unorm',
      blend: {
        color: component === 'color' ? testBlendComponent : defaultBlendComponent,
        alpha: component === 'alpha' ? testBlendComponent : defaultBlendComponent
      },
      writeMask
    }],

    fragmentShaderCode: getFragmentShaderCodeWithOutput(
      [{ values, plainType: 'f32', componentCount: 4 }],
      null,
      useBlendSrc1
    )
  });

  const _success = !isDualSourceBlendingFactor(blendFactor) || useBlendSrc1;
  const isAsync = false;
  vtu.doCreateRenderPipelineTest(t, isAsync, _success, descriptor);
});