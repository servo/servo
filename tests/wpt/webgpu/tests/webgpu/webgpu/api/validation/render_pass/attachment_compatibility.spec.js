/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation for attachment compatibility between render passes, bundles, and pipelines

TODO(#3363): Make this into a MaxLimitTest and increase kMaxColorAttachments.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { range } from '../../../../common/util/util.js';
import { getDefaultLimits, kTextureSampleCounts } from '../../../capability_info.js';
import {
  kSizedDepthStencilFormats,
  kUnsizedDepthStencilFormats,
  filterFormatsByFeature,
  getFeaturesForFormats,
  isDepthTextureFormat,
  isStencilTextureFormat,
  kPossibleColorRenderableTextureFormats } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';

// MAINTENANCE_TODO: This should be changed to kMaxColorAttachmentsToTest
// when this is made a MaxLimitTest (see above).
const kMaxColorAttachments = getDefaultLimits('core').maxColorAttachments.default;

const kColorAttachmentCounts = range(kMaxColorAttachments, (i) => i + 1);
const kColorAttachments = kColorAttachmentCounts.
map((count) => {
  // generate cases with 0..1 null attachments at different location
  // e.g. count == 2
  // [
  //    [1, 1],
  //    [0, 1],
  //    [1, 0],
  // ]
  // 0 (false) means null attachment, 1 (true) means non-null attachment, at the slot

  // Special cases: we need at least a color attachment, when we don't have depth stencil attachment
  if (count === 1) {
    return [[1]];
  }
  if (count === 2) {
    return [
    [1, 1],
    [0, 1],
    [1, 0]];

  }

  // [1, 1, ..., 1]: all color attachment are used
  let result = [new Array(count).fill(true)];

  // [1, 0, 1, ..., 1]: generate cases with one null attachment at different locations
  result = result.concat(
    range(count, (i) => {
      const r = new Array(count).fill(true);
      r[i] = false;
      return r;
    })
  );

  // [1, 0, 1, ..., 0, 1]: generate cases with two null attachments at different locations
  // To reduce test run time, limit the attachment count to <= 4
  if (count <= 4) {
    result = result.concat(
      range(count - 1, (i) => {
        const cases = [];
        for (let j = i + 1; j < count; j++) {
          const r = new Array(count).fill(true);
          r[i] = false;
          r[j] = false;
          cases.push(r);
        }
        return cases;
      }).flat()
    );
  }

  return result;
}).
flat();

const kDepthStencilAttachmentFormats = [
undefined,
...kSizedDepthStencilFormats,
...kUnsizedDepthStencilFormats];


const kFeaturesForDepthStencilAttachmentFormats = getFeaturesForFormats([
...kSizedDepthStencilFormats,
...kUnsizedDepthStencilFormats]
);

class F extends AllFeaturesMaxLimitsGPUTest {
  createAttachmentTextureView(format, sampleCount) {
    return this.createTextureTracked({
      // Size matching the "arbitrary" size used by ValidationTest helpers.
      size: [16, 16, 1],
      format,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      sampleCount
    }).createView();
  }

  createColorAttachment(
  format,
  sampleCount)
  {
    return format === null ?
    null :
    {
      view: this.createAttachmentTextureView(format, sampleCount),
      clearValue: [0, 0, 0, 0],
      loadOp: 'clear',
      storeOp: 'store'
    };
  }

  createDepthAttachment(
  format,
  sampleCount)
  {
    const attachment = {
      view: this.createAttachmentTextureView(format, sampleCount)
    };
    if (isDepthTextureFormat(format)) {
      attachment.depthClearValue = 0;
      attachment.depthLoadOp = 'clear';
      attachment.depthStoreOp = 'discard';
    }
    if (isStencilTextureFormat(format)) {
      attachment.stencilClearValue = 1;
      attachment.stencilLoadOp = 'clear';
      attachment.stencilStoreOp = 'discard';
    }
    return attachment;
  }

  createRenderPipeline(
  targets,
  depthStencil,
  sampleCount,
  cullMode)
  {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: `
            @vertex fn main() -> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 0.0);
            }`
        }),
        entryPoint: 'main'
      },
      fragment: {
        module: this.device.createShaderModule({
          code: '@fragment fn main() {}'
        }),
        entryPoint: 'main',
        targets
      },
      primitive: { topology: 'triangle-list', cullMode },
      depthStencil,
      multisample: { count: sampleCount }
    });
  }
}

export const g = makeTestGroup(F);

g.test('render_pass_and_bundle,color_format').
desc('Test that color attachment formats in render passes and bundles must match.').
paramsSubcasesOnly((u) =>
u //
.combine('passFormat', kPossibleColorRenderableTextureFormats).
combine('bundleFormat', kPossibleColorRenderableTextureFormats)
).
fn((t) => {
  const { passFormat, bundleFormat } = t.params;
  t.skipIfTextureFormatNotSupported(passFormat, bundleFormat);
  t.skipIfTextureFormatNotUsableAsRenderAttachment(passFormat, bundleFormat);

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: [bundleFormat]
  });
  const bundle = bundleEncoder.finish();

  const { encoder, validateFinishAndSubmit } = t.createEncoder('non-pass');
  const pass = encoder.beginRenderPass({
    colorAttachments: [t.createColorAttachment(passFormat)]
  });
  pass.executeBundles([bundle]);
  pass.end();
  validateFinishAndSubmit(passFormat === bundleFormat, true);
});

g.test('render_pass_and_bundle,color_count').
desc(
  `
  Test that the number of color attachments in render passes and bundles must match.
  `
).
paramsSubcasesOnly((u) =>
u //
.combine('passCount', kColorAttachmentCounts).
combine('bundleCount', kColorAttachmentCounts)
).
fn((t) => {
  const { passCount, bundleCount } = t.params;

  const { maxColorAttachments } = t.device.limits;
  t.skipIf(
    passCount > maxColorAttachments,
    `passCount: ${passCount} > maxColorAttachments for device: ${maxColorAttachments}`
  );
  t.skipIf(
    bundleCount > maxColorAttachments,
    `bundleCount: ${bundleCount} > maxColorAttachments for device: ${maxColorAttachments}`
  );

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: range(bundleCount, () => 'rgba8uint')
  });
  const bundle = bundleEncoder.finish();

  const { encoder, validateFinishAndSubmit } = t.createEncoder('non-pass');
  const pass = encoder.beginRenderPass({
    colorAttachments: range(passCount, () => t.createColorAttachment('rgba8uint'))
  });
  pass.executeBundles([bundle]);
  pass.end();
  validateFinishAndSubmit(passCount === bundleCount, true);
});

g.test('render_pass_and_bundle,color_sparse').
desc(
  `
  Test that each of color attachments in render passes and bundles must match.
  `
).
params((u) =>
u //
// introduce attachmentCount to make it easier to split the test
.combine('attachmentCount', kColorAttachmentCounts).
beginSubcases()
// Indices into kColorAttachments
.expand('iPass', (p) =>
range(kColorAttachments.length, (i) => i).filter(
  (i) => kColorAttachments[i].length === p.attachmentCount
)
).
expand('iBundle', (p) =>
range(kColorAttachments.length, (i) => i).filter(
  (i) => kColorAttachments[i].length === p.attachmentCount
)
)
).
fn((t) => {
  const passAttachments = kColorAttachments[t.params.iPass];
  const bundleAttachments = kColorAttachments[t.params.iBundle];

  const { maxColorAttachments } = t.device.limits;
  t.skipIf(passAttachments.length > maxColorAttachments);
  t.skipIf(bundleAttachments.length > maxColorAttachments);

  const colorFormats = bundleAttachments.map((i) => i ? 'rgba8uint' : null);
  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats
  });
  const bundle = bundleEncoder.finish();

  const { encoder, validateFinishAndSubmit } = t.createEncoder('non-pass');
  const colorAttachments = passAttachments.map((i) =>
  t.createColorAttachment(i ? 'rgba8uint' : null)
  );
  const pass = encoder.beginRenderPass({
    colorAttachments
  });
  pass.executeBundles([bundle]);
  pass.end();
  validateFinishAndSubmit(
    passAttachments.every((v, i) => v === bundleAttachments[i]),
    true
  );
});

g.test('render_pass_and_bundle,depth_format').
desc('Test that the depth attachment format in render passes and bundles must match.').
params((u) =>
u //
.combine('passFeature', kFeaturesForDepthStencilAttachmentFormats).
combine('bundleFeature', kFeaturesForDepthStencilAttachmentFormats).
beginSubcases().
expand('passFormat', ({ passFeature }) =>
filterFormatsByFeature(passFeature, kDepthStencilAttachmentFormats)
).
expand('bundleFormat', ({ bundleFeature }) =>
filterFormatsByFeature(bundleFeature, kDepthStencilAttachmentFormats)
)
).
fn((t) => {
  const { passFormat, bundleFormat } = t.params;
  t.skipIfTextureFormatNotSupported(passFormat, bundleFormat);

  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm'],
    depthStencilFormat: bundleFormat
  });
  const bundle = bundleEncoder.finish();

  const { encoder, validateFinishAndSubmit } = t.createEncoder('non-pass');
  const pass = encoder.beginRenderPass({
    colorAttachments: [t.createColorAttachment('rgba8unorm')],
    depthStencilAttachment:
    passFormat !== undefined ? t.createDepthAttachment(passFormat) : undefined
  });
  pass.executeBundles([bundle]);
  pass.end();
  validateFinishAndSubmit(passFormat === bundleFormat, true);
});

g.test('render_pass_and_bundle,sample_count').
desc('Test that the sample count in render passes and bundles must match.').
paramsSubcasesOnly((u) =>
u //
.combine('renderSampleCount', kTextureSampleCounts).
combine('bundleSampleCount', kTextureSampleCounts)
).
fn((t) => {
  const { renderSampleCount, bundleSampleCount } = t.params;
  const bundleEncoder = t.device.createRenderBundleEncoder({
    colorFormats: ['rgba8unorm'],
    sampleCount: bundleSampleCount
  });
  const bundle = bundleEncoder.finish();
  const { encoder, validateFinishAndSubmit } = t.createEncoder('non-pass');
  const pass = encoder.beginRenderPass({
    colorAttachments: [t.createColorAttachment('rgba8unorm', renderSampleCount)]
  });
  pass.executeBundles([bundle]);
  pass.end();
  validateFinishAndSubmit(renderSampleCount === bundleSampleCount, true);
});

g.test('render_pass_and_bundle,device_mismatch').
desc('Test that render passes cannot be called with bundles created from another device.').
paramsSubcasesOnly((u) => u.combine('mismatched', [true, false])).
beforeAllSubcases((t) => t.usesMismatchedDevice()).
fn((t) => {
  const { mismatched } = t.params;
  const sourceDevice = mismatched ? t.mismatchedDevice : t.device;

  const format = 'r16uint';
  const bundleEncoder = sourceDevice.createRenderBundleEncoder({
    colorFormats: [format]
  });
  const bundle = bundleEncoder.finish();

  const { encoder, validateFinishAndSubmit } = t.createEncoder('non-pass');
  const pass = encoder.beginRenderPass({
    colorAttachments: [t.createColorAttachment(format)]
  });
  pass.executeBundles([bundle]);
  pass.end();
  validateFinishAndSubmit(!mismatched, true);
});

g.test('render_pass_or_bundle_and_pipeline,color_format').
desc(
  `
Test that color attachment formats in render passes or bundles match the pipeline color format.
`
).
params((u) =>
u.
combine('encoderType', ['render pass', 'render bundle']).
beginSubcases().
combine('encoderFormat', kPossibleColorRenderableTextureFormats).
combine('pipelineFormat', kPossibleColorRenderableTextureFormats)
).
fn((t) => {
  const { encoderType, encoderFormat, pipelineFormat } = t.params;
  t.skipIfTextureFormatNotSupported(encoderFormat, pipelineFormat);
  t.skipIfTextureFormatNotUsableAsRenderAttachment(encoderFormat, pipelineFormat);

  const pipeline = t.createRenderPipeline([{ format: pipelineFormat, writeMask: 0 }]);

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType, {
    attachmentInfo: { colorFormats: [encoderFormat] }
  });
  encoder.setPipeline(pipeline);
  validateFinishAndSubmit(encoderFormat === pipelineFormat, true);
});

g.test('render_pass_or_bundle_and_pipeline,color_count').
desc(
  `
Test that the number of color attachments in render passes or bundles match the pipeline color
count.
`
).
params((u) =>
u.
combine('encoderType', ['render pass', 'render bundle']).
beginSubcases().
combine('encoderCount', kColorAttachmentCounts).
combine('pipelineCount', kColorAttachmentCounts)
).
fn((t) => {
  const { encoderType, encoderCount, pipelineCount } = t.params;

  const { maxColorAttachments } = t.device.limits;
  t.skipIf(
    pipelineCount > maxColorAttachments,
    `pipelineCount: ${pipelineCount} > maxColorAttachments for device: ${maxColorAttachments}`
  );
  t.skipIf(
    encoderCount > maxColorAttachments,
    `encoderCount: ${encoderCount} > maxColorAttachments for device: ${maxColorAttachments}`
  );

  const pipeline = t.createRenderPipeline(
    range(pipelineCount, () => ({ format: 'rgba8uint', writeMask: 0 }))
  );

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType, {
    attachmentInfo: { colorFormats: range(encoderCount, () => 'rgba8uint') }
  });
  encoder.setPipeline(pipeline);
  validateFinishAndSubmit(encoderCount === pipelineCount, true);
});

g.test('render_pass_or_bundle_and_pipeline,color_sparse').
desc(
  `
Test that each of color attachments in render passes or bundles match that of the pipeline.
`
).
params((u) =>
u.
combine('encoderType', ['render pass', 'render bundle'])
// introduce attachmentCount to make it easier to split the test
.combine('attachmentCount', kColorAttachmentCounts).
beginSubcases()
// Indices into kColorAttachments
.expand('iEncoder', (p) =>
range(kColorAttachments.length, (i) => i).filter(
  (i) => kColorAttachments[i].length === p.attachmentCount
)
).
expand('iPipeline', (p) =>
range(kColorAttachments.length, (i) => i).filter(
  (i) => kColorAttachments[i].length === p.attachmentCount
)
)
).
fn((t) => {
  const { encoderType } = t.params;
  const encoderAttachments = kColorAttachments[t.params.iEncoder];
  const pipelineAttachments = kColorAttachments[t.params.iPipeline];

  const { maxColorAttachments } = t.device.limits;
  t.skipIf(encoderAttachments.length > maxColorAttachments);
  t.skipIf(pipelineAttachments.length > maxColorAttachments);

  const colorTargets = pipelineAttachments.map((i) =>
  i ? { format: 'rgba8uint', writeMask: 0 } : null
  );
  const pipeline = t.createRenderPipeline(colorTargets);

  const colorFormats = encoderAttachments.map((i) => i ? 'rgba8uint' : null);
  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType, {
    attachmentInfo: { colorFormats }
  });
  encoder.setPipeline(pipeline);
  validateFinishAndSubmit(
    encoderAttachments.every((v, i) => v === pipelineAttachments[i]),
    true
  );
});

g.test('render_pass_or_bundle_and_pipeline,depth_format').
desc(
  `
Test that the depth attachment format in render passes or bundles match the pipeline depth format.
`
).
params((u) =>
u.
combine('encoderType', ['render pass', 'render bundle']).
combine('encoderFormatFeature', kFeaturesForDepthStencilAttachmentFormats).
combine('pipelineFormatFeature', kFeaturesForDepthStencilAttachmentFormats).
beginSubcases().
expand('encoderFormat', ({ encoderFormatFeature }) =>
filterFormatsByFeature(encoderFormatFeature, kDepthStencilAttachmentFormats)
).
expand('pipelineFormat', ({ pipelineFormatFeature }) =>
filterFormatsByFeature(pipelineFormatFeature, kDepthStencilAttachmentFormats)
)
).
fn((t) => {
  const { encoderType, encoderFormat, pipelineFormat } = t.params;
  t.skipIfTextureFormatNotSupported(encoderFormat, pipelineFormat);

  const pipeline = t.createRenderPipeline(
    [{ format: 'rgba8unorm', writeMask: 0 }],
    pipelineFormat !== undefined ?
    { format: pipelineFormat, depthCompare: 'always', depthWriteEnabled: false } :
    undefined
  );

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType, {
    attachmentInfo: { colorFormats: ['rgba8unorm'], depthStencilFormat: encoderFormat }
  });
  encoder.setPipeline(pipeline);
  validateFinishAndSubmit(encoderFormat === pipelineFormat, true);
});

const kStencilFaceStates = [
{ failOp: 'keep', depthFailOp: 'keep', passOp: 'keep' },
{ failOp: 'zero', depthFailOp: 'zero', passOp: 'zero' }];


g.test('render_pass_or_bundle_and_pipeline,depth_stencil_read_only_write_state').
desc(
  `
Test that the depth stencil read only state in render passes or bundles is compatible with the depth stencil write state of the pipeline.
`
).
params((u) =>
u.
combine('encoderType', ['render pass', 'render bundle']).
combine('format', kDepthStencilAttachmentFormats).
beginSubcases()
// pass/bundle state
.combine('depthReadOnly', [false, true]).
combine('stencilReadOnly', [false, true]).
combine('stencilFront', kStencilFaceStates).
combine('stencilBack', kStencilFaceStates)
// pipeline state
.combine('depthWriteEnabled', [false, true]).
combine('stencilWriteMask', [0, 0xffffffff]).
combine('cullMode', ['none', 'front', 'back']).
filter((p) => {
  if (p.format) {
    // If the format has no depth aspect, the depthReadOnly, depthWriteEnabled of the pipeline must not be true
    // in order to create a valid render pipeline.
    if (!isDepthTextureFormat(p.format) && p.depthWriteEnabled) {
      return false;
    }
    // If the format has no stencil aspect, the stencil state operation must be 'keep'
    // in order to create a valid render pipeline.
    if (
    !isStencilTextureFormat(p.format) && (
    p.stencilFront.failOp !== 'keep' || p.stencilBack.failOp !== 'keep'))
    {
      return false;
    }
  }
  // No depthStencil attachment
  return true;
})
).
fn((t) => {
  const {
    encoderType,
    format,
    depthReadOnly,
    stencilReadOnly,
    depthWriteEnabled,
    stencilWriteMask,
    cullMode,
    stencilFront,
    stencilBack
  } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const pipeline = t.createRenderPipeline(
    [{ format: 'rgba8unorm', writeMask: 0 }],
    format === undefined ?
    undefined :
    {
      format,
      depthWriteEnabled,
      depthCompare: 'always',
      stencilWriteMask,
      stencilFront,
      stencilBack
    },
    1,
    cullMode
  );

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType, {
    attachmentInfo: {
      colorFormats: ['rgba8unorm'],
      depthStencilFormat: format,
      depthReadOnly,
      stencilReadOnly
    }
  });
  encoder.setPipeline(pipeline);

  let writesDepth = false;
  let writesStencil = false;
  if (format) {
    writesDepth = depthWriteEnabled;
    if (stencilWriteMask !== 0) {
      if (
      cullMode !== 'front' && (
      stencilFront.passOp !== 'keep' ||
      stencilFront.depthFailOp !== 'keep' ||
      stencilFront.failOp !== 'keep'))
      {
        writesStencil = true;
      }
      if (
      cullMode !== 'back' && (
      stencilBack.passOp !== 'keep' ||
      stencilBack.depthFailOp !== 'keep' ||
      stencilBack.failOp !== 'keep'))
      {
        writesStencil = true;
      }
    }
  }

  let isValid = true;
  if (writesDepth) {
    isValid &&= !depthReadOnly;
  }
  if (writesStencil) {
    isValid &&= !stencilReadOnly;
  }

  validateFinishAndSubmit(isValid, true);
});

g.test('render_pass_or_bundle_and_pipeline,sample_count').
desc(
  `
Test that the sample count in render passes or bundles match the pipeline sample count for both color texture and depthstencil texture.
`
).
params((u) =>
u.
combine('encoderType', ['render pass', 'render bundle']).
combine('attachmentType', ['color', 'depthstencil']).
beginSubcases().
combine('encoderSampleCount', kTextureSampleCounts).
combine('pipelineSampleCount', kTextureSampleCounts)
).
fn((t) => {
  const { encoderType, attachmentType, encoderSampleCount, pipelineSampleCount } = t.params;

  const colorFormats = attachmentType === 'color' ? ['rgba8unorm'] : [];
  const depthStencilFormat =
  attachmentType === 'depthstencil' ? 'depth24plus-stencil8' : undefined;

  const pipeline = t.createRenderPipeline(
    colorFormats.map((format) => ({ format, writeMask: 0 })),
    depthStencilFormat ?
    { format: depthStencilFormat, depthWriteEnabled: false, depthCompare: 'always' } :
    undefined,
    pipelineSampleCount
  );

  const { encoder, validateFinishAndSubmit } = t.createEncoder(encoderType, {
    attachmentInfo: { colorFormats, depthStencilFormat, sampleCount: encoderSampleCount }
  });
  encoder.setPipeline(pipeline);
  validateFinishAndSubmit(encoderSampleCount === pipelineSampleCount, true);
});