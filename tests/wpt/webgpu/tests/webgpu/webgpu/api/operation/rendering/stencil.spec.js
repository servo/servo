/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test related to stencil states, stencil op, compare func, etc.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import {
  kBufferSizeAlignment,
  kMinDynamicBufferOffsetAlignment } from
'../../../capability_info.js';
import {
  kStencilTextureFormats,

  isDepthTextureFormat,
  isStencilTextureFormat,
  kDepthStencilFormats,
  depthStencilFormatAspectSize } from
'../../../format_info.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../../gpu_test.js';
import * as ttu from '../../../texture_test_utils.js';
import { align } from '../../../util/math.js';
import { DataArrayGenerator } from '../../../util/texture/data_generation.js';
import { dataBytesForCopyOrFail, getTextureCopyLayout } from '../../../util/texture/layout.js';
import { TexelView } from '../../../util/texture/texel_view.js';

const kBaseColor = new Float32Array([1.0, 1.0, 1.0, 1.0]);
const kRedStencilColor = new Float32Array([1.0, 0.0, 0.0, 1.0]);
const kGreenStencilColor = new Float32Array([0.0, 1.0, 0.0, 1.0]);







class StencilTest extends AllFeaturesMaxLimitsGPUTest {
  checkStencilOperation(
  depthStencilFormat,
  testStencilState,
  initialStencil,
  _expectedStencil,
  depthCompare = 'always')
  {
    const kReferenceStencil = 3;

    const baseStencilState = {
      compare: 'always',
      failOp: 'keep',
      passOp: 'replace'
    };

    const stencilState = {
      compare: 'equal',
      failOp: 'keep',
      passOp: 'keep'
    };

    const baseState = {
      format: depthStencilFormat,
      depthWriteEnabled: false,
      depthCompare: 'always',
      stencilFront: baseStencilState,
      stencilBack: baseStencilState
    };

    const testState = {
      format: depthStencilFormat,
      depthWriteEnabled: false,
      depthCompare,
      stencilFront: testStencilState,
      stencilBack: testStencilState
    };

    const testState2 = {
      format: depthStencilFormat,
      depthWriteEnabled: false,
      depthCompare: 'always',
      stencilFront: stencilState,
      stencilBack: stencilState
    };

    const testStates = [
    // Draw the base triangle with stencil reference 1. This clears the stencil buffer to 1.
    { state: baseState, color: kBaseColor, stencil: initialStencil },
    { state: testState, color: kRedStencilColor, stencil: kReferenceStencil },
    { state: testState2, color: kGreenStencilColor, stencil: _expectedStencil }];

    this.runStencilStateTest(depthStencilFormat, testStates, kGreenStencilColor);
  }

  checkStencilCompareFunction(
  depthStencilFormat,
  compareFunction,
  stencilRefValue,
  expectedColor)
  {
    const baseStencilState = {
      compare: 'always',
      failOp: 'keep',
      passOp: 'replace'
    };

    const stencilState = {
      compare: compareFunction,
      failOp: 'keep',
      passOp: 'keep'
    };

    const baseState = {
      format: depthStencilFormat,
      depthWriteEnabled: false,
      depthCompare: 'always',
      stencilFront: baseStencilState,
      stencilBack: baseStencilState
    };

    const testState = {
      format: depthStencilFormat,
      depthWriteEnabled: false,
      depthCompare: 'always',
      stencilFront: stencilState,
      stencilBack: stencilState
    };

    const testStates = [
    // Draw the base triangle with stencil reference 1. This clears the stencil buffer to 1.
    { state: baseState, color: kBaseColor, stencil: 1 },
    { state: testState, color: kGreenStencilColor, stencil: stencilRefValue }];

    this.runStencilStateTest(depthStencilFormat, testStates, expectedColor);
  }

  runStencilStateTest(
  depthStencilFormat,
  testStates,
  expectedColor,
  isSingleEncoderMultiplePass = false)
  {
    const renderTargetFormat = 'rgba8unorm';
    const renderTarget = this.createTextureTracked({
      format: renderTargetFormat,
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    const depthTexture = this.createTextureTracked({
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      format: depthStencilFormat,
      sampleCount: 1,
      mipLevelCount: 1,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_DST
    });

    const hasDepth = isDepthTextureFormat(depthStencilFormat);
    const depthStencilAttachment = {
      view: depthTexture.createView(),
      depthLoadOp: hasDepth ? 'load' : undefined,
      depthStoreOp: hasDepth ? 'store' : undefined,
      stencilLoadOp: 'load',
      stencilStoreOp: 'store'
    };

    const encoder = this.device.createCommandEncoder();
    let pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: renderTarget.createView(),
        loadOp: 'load',
        storeOp: 'store'
      }],

      depthStencilAttachment
    });

    if (isSingleEncoderMultiplePass) {
      pass.end();
    }

    // Draw a triangle with the given stencil reference and the comparison function.
    // The color will be kGreenStencilColor if the stencil test passes, and kBaseColor if not.
    for (const test of testStates) {
      if (isSingleEncoderMultiplePass) {
        pass = encoder.beginRenderPass({
          colorAttachments: [
          {
            view: renderTarget.createView(),
            loadOp: 'load',
            storeOp: 'store'
          }],

          depthStencilAttachment
        });
      }
      const testPipeline = this.createRenderPipelineForTest(test.state);
      pass.setPipeline(testPipeline);
      if (test.stencil !== undefined) {
        pass.setStencilReference(test.stencil);
      }
      pass.setBindGroup(
        0,
        this.createBindGroupForTest(testPipeline.getBindGroupLayout(0), test.color)
      );
      pass.draw(1);

      if (isSingleEncoderMultiplePass) {
        pass.end();
      }
    }

    if (!isSingleEncoderMultiplePass) {
      pass.end();
    }
    this.device.queue.submit([encoder.finish()]);

    const expColor = {
      R: expectedColor[0],
      G: expectedColor[1],
      B: expectedColor[2],
      A: expectedColor[3]
    };
    const expTexelView = TexelView.fromTexelsAsColors(renderTargetFormat, (_coords) => expColor);
    ttu.expectTexelViewComparisonIsOkInTexture(
      this,
      { texture: renderTarget },
      expTexelView,
      [1, 1]
    );
  }

  createRenderPipelineForTest(depthStencil) {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: `
            @vertex
            fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
                return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }
            `
        }),
        entryPoint: 'main'
      },
      fragment: {
        targets: [{ format: 'rgba8unorm' }],
        module: this.device.createShaderModule({
          code: `
            struct Params {
              color : vec4<f32>
            }
            @group(0) @binding(0) var<uniform> params : Params;

            @fragment fn main() -> @location(0) vec4<f32> {
                return vec4<f32>(params.color);
            }`
        }),
        entryPoint: 'main'
      },
      primitive: { topology: 'point-list' },
      depthStencil
    });
  }

  createBindGroupForTest(layout, data) {
    return this.device.createBindGroup({
      layout,
      entries: [
      {
        binding: 0,
        resource: {
          buffer: this.makeBufferWithContents(data, GPUBufferUsage.UNIFORM)
        }
      }]

    });
  }
}

export const g = makeTestGroup(StencilTest);

g.test('stencil_compare_func').
desc(
  `
  Tests that stencil comparison functions with the stencil reference value works as expected.
  `
).
params((u) =>
u //
.combine('format', kStencilTextureFormats).
combineWithParams([
{ stencilCompare: 'always', stencilRefValue: 0, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'always', stencilRefValue: 1, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'always', stencilRefValue: 2, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'equal', stencilRefValue: 0, _expectedColor: kBaseColor },
{ stencilCompare: 'equal', stencilRefValue: 1, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'equal', stencilRefValue: 2, _expectedColor: kBaseColor },
{ stencilCompare: 'greater', stencilRefValue: 0, _expectedColor: kBaseColor },
{ stencilCompare: 'greater', stencilRefValue: 1, _expectedColor: kBaseColor },
{ stencilCompare: 'greater', stencilRefValue: 2, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'greater-equal', stencilRefValue: 0, _expectedColor: kBaseColor },
{ stencilCompare: 'greater-equal', stencilRefValue: 1, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'greater-equal', stencilRefValue: 2, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'less', stencilRefValue: 0, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'less', stencilRefValue: 1, _expectedColor: kBaseColor },
{ stencilCompare: 'less', stencilRefValue: 2, _expectedColor: kBaseColor },
{ stencilCompare: 'less-equal', stencilRefValue: 0, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'less-equal', stencilRefValue: 1, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'less-equal', stencilRefValue: 2, _expectedColor: kBaseColor },
{ stencilCompare: 'never', stencilRefValue: 0, _expectedColor: kBaseColor },
{ stencilCompare: 'never', stencilRefValue: 1, _expectedColor: kBaseColor },
{ stencilCompare: 'never', stencilRefValue: 2, _expectedColor: kBaseColor },
{ stencilCompare: 'not-equal', stencilRefValue: 0, _expectedColor: kGreenStencilColor },
{ stencilCompare: 'not-equal', stencilRefValue: 1, _expectedColor: kBaseColor },
{ stencilCompare: 'not-equal', stencilRefValue: 2, _expectedColor: kGreenStencilColor }]
)
).
fn((t) => {
  const { format, stencilCompare, stencilRefValue, _expectedColor } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  t.checkStencilCompareFunction(format, stencilCompare, stencilRefValue, _expectedColor);
});

g.test('stencil_passOp_operation').
desc(
  `
  Test that the stencil operation is executed on stencil pass. A triangle is drawn with the 'always'
  comparison function, so it should pass. Then, test that each pass stencil operation works with the
  given stencil values correctly as expected. For example,
    - If the pass operation is 'keep', it keeps the initial stencil value.
    - If the pass operation is 'replace', it replaces the initial stencil value with the reference
      stencil value.
  `
).
params((u) =>
u //
.combine('format', kStencilTextureFormats).
combineWithParams([
{ passOp: 'keep', initialStencil: 1, _expectedStencil: 1 },
{ passOp: 'zero', initialStencil: 1, _expectedStencil: 0 },
{ passOp: 'replace', initialStencil: 1, _expectedStencil: 3 },
{ passOp: 'invert', initialStencil: 0xf0, _expectedStencil: 0x0f },
{ passOp: 'increment-clamp', initialStencil: 1, _expectedStencil: 2 },
{ passOp: 'increment-clamp', initialStencil: 0xff, _expectedStencil: 0xff },
{ passOp: 'increment-wrap', initialStencil: 1, _expectedStencil: 2 },
{ passOp: 'increment-wrap', initialStencil: 0xff, _expectedStencil: 0 },
{ passOp: 'decrement-clamp', initialStencil: 1, _expectedStencil: 0 },
{ passOp: 'decrement-clamp', initialStencil: 0, _expectedStencil: 0 },
{ passOp: 'decrement-wrap', initialStencil: 1, _expectedStencil: 0 },
{ passOp: 'decrement-wrap', initialStencil: 0, _expectedStencil: 0xff }]
)
).
fn((t) => {
  const { format, passOp, initialStencil, _expectedStencil } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const stencilState = {
    compare: 'always',
    failOp: 'keep',
    passOp
  };

  t.checkStencilOperation(format, stencilState, initialStencil, _expectedStencil);
});

g.test('stencil_failOp_operation').
desc(
  `
  Test that the stencil operation is executed on stencil fail. A triangle is drawn with the 'never'
  comparison function, so it should fail. Then, test that each fail stencil operation works with the
  given stencil values correctly as expected. For example,
    - If the fail operation is 'keep', it keeps the initial stencil value.
    - If the fail operation is 'replace', it replaces the initial stencil value with the reference
      stencil value.
  `
).
params((u) =>
u //
.combine('format', kStencilTextureFormats).
combineWithParams([
{ failOp: 'keep', initialStencil: 1, _expectedStencil: 1 },
{ failOp: 'zero', initialStencil: 1, _expectedStencil: 0 },
{ failOp: 'replace', initialStencil: 1, _expectedStencil: 3 },
{ failOp: 'invert', initialStencil: 0xf0, _expectedStencil: 0x0f },
{ failOp: 'increment-clamp', initialStencil: 1, _expectedStencil: 2 },
{ failOp: 'increment-clamp', initialStencil: 0xff, _expectedStencil: 0xff },
{ failOp: 'increment-wrap', initialStencil: 1, _expectedStencil: 2 },
{ failOp: 'increment-wrap', initialStencil: 0xff, _expectedStencil: 0 },
{ failOp: 'decrement-clamp', initialStencil: 1, _expectedStencil: 0 },
{ failOp: 'decrement-clamp', initialStencil: 0, _expectedStencil: 0 },
{ failOp: 'decrement-wrap', initialStencil: 2, _expectedStencil: 1 },
{ failOp: 'decrement-wrap', initialStencil: 1, _expectedStencil: 0 },
{ failOp: 'decrement-wrap', initialStencil: 0, _expectedStencil: 0xff }]
)
).
fn((t) => {
  const { format, failOp, initialStencil, _expectedStencil } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const stencilState = {
    compare: 'never',
    failOp,
    passOp: 'keep'
  };

  // Draw the base triangle with stencil reference 1. This clears the stencil buffer to 1.
  // Always fails because the comparison never passes. Therefore red is never drawn, and the
  // stencil contents may be updated according to `operation`.
  t.checkStencilOperation(format, stencilState, initialStencil, _expectedStencil);
});

g.test('stencil_depthFailOp_operation').
desc(
  `
  Test that the stencil operation is executed on depthCompare fail. A triangle is drawn with the
  'never' depthCompare, so it should fail the depth test. Then, test that each 'depthFailOp' stencil operation
  works with the given stencil values correctly as expected. For example,
    - If the depthFailOp operation is 'keep', it keeps the initial stencil value.
    - If the depthFailOp operation is 'replace', it replaces the initial stencil value with the
      reference stencil value.
  `
).
params((u) =>
u //
.combine(
  'format',
  kDepthStencilFormats.filter(
    (format) => isDepthTextureFormat(format) && isStencilTextureFormat(format)
  )
).
combineWithParams([
{ depthFailOp: 'keep', initialStencil: 1, _expectedStencil: 1 },
{ depthFailOp: 'zero', initialStencil: 1, _expectedStencil: 0 },
{ depthFailOp: 'replace', initialStencil: 1, _expectedStencil: 3 },
{ depthFailOp: 'invert', initialStencil: 0xf0, _expectedStencil: 0x0f },
{ depthFailOp: 'increment-clamp', initialStencil: 1, _expectedStencil: 2 },
{ depthFailOp: 'increment-clamp', initialStencil: 0xff, _expectedStencil: 0xff },
{ depthFailOp: 'increment-wrap', initialStencil: 1, _expectedStencil: 2 },
{ depthFailOp: 'increment-wrap', initialStencil: 0xff, _expectedStencil: 0 },
{ depthFailOp: 'decrement-clamp', initialStencil: 1, _expectedStencil: 0 },
{ depthFailOp: 'decrement-clamp', initialStencil: 0, _expectedStencil: 0 },
{ depthFailOp: 'decrement-wrap', initialStencil: 2, _expectedStencil: 1 },
{ depthFailOp: 'decrement-wrap', initialStencil: 1, _expectedStencil: 0 },
{ depthFailOp: 'decrement-wrap', initialStencil: 0, _expectedStencil: 0xff }]
)
).
fn((t) => {
  const { format, depthFailOp, initialStencil, _expectedStencil } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const stencilState = {
    compare: 'always',
    failOp: 'keep',
    passOp: 'keep',
    depthFailOp
  };

  // Call checkStencilOperation function with enabling the depthTest to test that the depthFailOp
  // stencil operation works as expected.
  t.checkStencilOperation(format, stencilState, initialStencil, _expectedStencil, 'never');
});

g.test('stencil_read_write_mask').
desc(
  `
  Tests that setting a stencil read/write masks work. Basically, The base triangle sets 3 to the
  stencil, and then try to draw a triangle with different stencil values.
    - In case that 'write' mask is 1,
      * If the stencil of the triangle is 1, it draws because
        'base stencil(3) & write mask(1) == triangle stencil(1)'.
      * If the stencil of the triangle is 2, it does not draw because
        'base stencil(3) & write mask(1) != triangle stencil(2)'.

    - In case that 'read' mask is 2,
      * If the stencil of the triangle is 1, it does not draw because
        'base stencil(3) & read mask(2) != triangle stencil(1)'.
      * If the stencil of the triangle is 2, it draws because
        'base stencil(3) & read mask(2) == triangle stencil(2)'.
  `
).
params((u) =>
u //
.combine('format', kStencilTextureFormats).
combineWithParams([
{ maskType: 'write', stencilRefValue: 1, _expectedColor: kRedStencilColor },
{ maskType: 'write', stencilRefValue: 2, _expectedColor: kBaseColor },
{ maskType: 'read', stencilRefValue: 1, _expectedColor: kBaseColor },
{ maskType: 'read', stencilRefValue: 2, _expectedColor: kRedStencilColor }]
)
).
fn((t) => {
  const { format, maskType, stencilRefValue, _expectedColor } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const baseStencilState = {
    compare: 'always',
    failOp: 'keep',
    passOp: 'replace'
  };

  const stencilState = {
    compare: 'equal',
    failOp: 'keep',
    passOp: 'keep'
  };

  const baseState = {
    format,
    depthWriteEnabled: false,
    depthCompare: 'always',
    stencilFront: baseStencilState,
    stencilBack: baseStencilState,
    stencilReadMask: 0xff,
    stencilWriteMask: maskType === 'write' ? 0x1 : 0xff
  };

  const testState = {
    format,
    depthWriteEnabled: false,
    depthCompare: 'always',
    stencilFront: stencilState,
    stencilBack: stencilState,
    stencilReadMask: maskType === 'read' ? 0x2 : 0xff,
    stencilWriteMask: 0xff
  };

  const testStates = [
  // Draw the base triangle with stencil reference 3. This clears the stencil buffer to 3.
  { state: baseState, color: kBaseColor, stencil: 3 },
  { state: testState, color: kRedStencilColor, stencil: stencilRefValue }];


  t.runStencilStateTest(format, testStates, _expectedColor);
});

g.test('stencil_reference_initialized').
desc('Test that stencil reference is initialized as zero for new render pass.').
params((u) => u.combine('format', kStencilTextureFormats)).
fn((t) => {
  const { format } = t.params;
  t.skipIfTextureFormatNotSupported(format);

  const baseStencilState = {
    compare: 'always',
    passOp: 'replace'
  };

  const testStencilState = {
    compare: 'equal',
    passOp: 'keep'
  };

  const hasDepth = isDepthTextureFormat(format);

  const baseState = {
    format,
    depthWriteEnabled: hasDepth,
    depthCompare: 'always',
    stencilFront: baseStencilState,
    stencilBack: baseStencilState
  };

  const testState = {
    format,
    depthWriteEnabled: hasDepth,
    depthCompare: 'always',
    stencilFront: testStencilState,
    stencilBack: testStencilState
  };

  // First pass sets the stencil to 0x1, the second pass sets the stencil to its default
  // value, and the third pass tests if the stencil is zero.
  const testStates = [
  { state: baseState, color: kBaseColor, stencil: 0x1 },
  { state: baseState, color: kRedStencilColor, stencil: undefined },
  { state: testState, color: kGreenStencilColor, stencil: 0x0 }];


  // The third draw should pass the stencil test since the second pass set it to default zero.
  t.runStencilStateTest(format, testStates, kGreenStencilColor, true);
});

const dataGenerator = new DataArrayGenerator();
g.test('stencil_accumulation').
desc(
  `A variation of a technique previously used to verify stencil texture copy that was observed to
    fail on some Qualcomm chipsets. Attempts to read back from a stencil texture by doing multiple
    fullscreen renders, one per stencil bit. Each pass sets the stencil mask and stencil reference
    to 1 << passIndex, and renders 1 << passIndex into the red channel with additive blending. This
    should duplicate the stencil values into the red channel of the texture, and on most GPUs it
    works. Technique is replicated here to catch driver bugs, while the original stencil copy test
    has been updated to verify the stencil values using copyTextureToBuffer, which is more
    straightforward and allows those tests to focus on the behavior of the APIs in question.
`
).
params((u) =>
u.
combine('format', kDepthStencilFormats).
filter((t) => isStencilTextureFormat(t.format)).
beginSubcases().
combineWithParams([
{ offsetInBlocks: 0, dataPaddingInBytes: 0 },
{ offsetInBlocks: 16, dataPaddingInBytes: 0 },
{ offsetInBlocks: 768, dataPaddingInBytes: 0 },
{ offsetInBlocks: 0, dataPaddingInBytes: 1 }]
).
combine('copyDepth', [1, 2]).
combine('mipLevel', [0, 2])
).
fn((t) => {
  const { format, offsetInBlocks, dataPaddingInBytes, copyDepth, mipLevel } = t.params;
  t.skipIfTextureFormatNotSupported(format);
  const aspect = 'stencil-only';
  const bytesPerBlock = depthStencilFormatAspectSize(format, aspect);
  const initialDataOffset = offsetInBlocks * bytesPerBlock;
  const copySize = [3, 3, copyDepth];
  const rowsPerImage = 3;
  const bytesPerRow = 256;

  const textureSize = [copySize[0] << mipLevel, copySize[1] << mipLevel, copyDepth];

  const minDataSize = dataBytesForCopyOrFail({
    layout: { offset: initialDataOffset, bytesPerRow, rowsPerImage },
    format: 'stencil8',
    copySize,
    method: 'WriteTexture'
  });
  const initialDataSize = minDataSize + dataPaddingInBytes;

  const srcTexture = t.createTextureTracked({
    size: textureSize,
    usage:
    GPUTextureUsage.COPY_DST | GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT,
    format,
    mipLevelCount: mipLevel + 1
  });

  const writeSize = [textureSize[0] >> mipLevel, textureSize[1] >> mipLevel, textureSize[2]];
  const initialData = dataGenerator.generateView(
    align(initialDataSize, kBufferSizeAlignment),
    0,
    initialDataOffset
  );

  t.queue.writeTexture(
    { texture: srcTexture, aspect: 'stencil-only', mipLevel },
    initialData,
    {
      offset: initialDataOffset,
      bytesPerRow,
      rowsPerImage
    },
    writeSize
  );

  // Everything below here was previously checkStencilTextureContent in image_copy.spec.ts
  const stencilBitCount = 8;

  // Prepare the uniform buffer that stores the bit indices (from 0 to 7) at stride 256 (required
  // by Dynamic Buffer Offset).
  const uniformBufferSize = kMinDynamicBufferOffsetAlignment * (stencilBitCount - 1) + 4;
  const uniformBufferData = new Uint32Array(uniformBufferSize / 4);
  for (let i = 1; i < stencilBitCount; ++i) {
    uniformBufferData[kMinDynamicBufferOffsetAlignment / 4 * i] = i;
  }
  const uniformBuffer = t.makeBufferWithContents(
    uniformBufferData,
    GPUBufferUsage.COPY_DST | GPUBufferUsage.UNIFORM
  );

  // Prepare the base render pipeline descriptor (all the settings expect stencilReadMask).
  const bindGroupLayout = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      buffer: {
        type: 'uniform',
        minBindingSize: 4,
        hasDynamicOffset: true
      }
    }]

  });
  const renderPipelineDescriptorBase = {
    layout: t.device.createPipelineLayout({ bindGroupLayouts: [bindGroupLayout] }),
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex
            fn main(@builtin(vertex_index) VertexIndex : u32)-> @builtin(position) vec4<f32> {
              var pos : array<vec2<f32>, 6> = array<vec2<f32>, 6>(
                  vec2<f32>(-1.0,  1.0),
                  vec2<f32>(-1.0, -1.0),
                  vec2<f32>( 1.0,  1.0),
                  vec2<f32>(-1.0, -1.0),
                  vec2<f32>( 1.0,  1.0),
                  vec2<f32>( 1.0, -1.0));
              return vec4<f32>(pos[VertexIndex], 0.0, 1.0);
            }`
      }),
      entryPoint: 'main'
    },

    fragment: {
      module: t.device.createShaderModule({
        code: `
            struct Params {
              stencilBitIndex: u32
            };
            @group(0) @binding(0) var<uniform> param: Params;
            @fragment
            fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(f32(1u << param.stencilBitIndex) / 255.0, 0.0, 0.0, 0.0);
            }`
      }),
      entryPoint: 'main',
      targets: [
      {
        // As we implement "rendering one bit in each draw() call" with blending operation
        // 'add', the format of outputTexture must support blending.
        format: 'r8unorm',
        blend: {
          color: { srcFactor: 'one', dstFactor: 'one', operation: 'add' },
          alpha: {}
        }
      }]

    },

    primitive: {
      topology: 'triangle-list'
    },

    depthStencil: {
      depthWriteEnabled: false,
      depthCompare: 'always',
      format,
      stencilFront: {
        compare: 'equal'
      },
      stencilBack: {
        compare: 'equal'
      }
    }
  };

  // Prepare the bindGroup that contains uniformBuffer and referenceTexture.
  const bindGroup = t.device.createBindGroup({
    layout: bindGroupLayout,
    entries: [
    {
      binding: 0,
      resource: {
        buffer: uniformBuffer,
        size: 4
      }
    }]

  });

  // "Copy" the stencil value into the color attachment with 8 draws in one render pass. Each draw
  // will "Copy" one bit of the stencil value into the color attachment. The bit of the stencil
  // value is specified by setStencilReference().
  const copyFromOutputTextureLayout = getTextureCopyLayout(
    format,
    '2d',
    [textureSize[0], textureSize[1], 1],
    {
      mipLevel,
      aspect: 'stencil-only'
    }
  );
  const outputTextureSize = [
  copyFromOutputTextureLayout.mipSize[0],
  copyFromOutputTextureLayout.mipSize[1],
  1];

  const outputTexture = t.createTextureTracked({
    format: 'r8unorm',
    size: outputTextureSize,
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });

  for (let stencilTextureLayer = 0; stencilTextureLayer < textureSize[2]; ++stencilTextureLayer) {
    const encoder = t.device.createCommandEncoder();
    const depthStencilAttachment = {
      view: srcTexture.createView({
        baseMipLevel: mipLevel,
        mipLevelCount: 1,
        baseArrayLayer: stencilTextureLayer,
        arrayLayerCount: 1
      })
    };
    if (isDepthTextureFormat(format)) {
      depthStencilAttachment.depthClearValue = 0;
      depthStencilAttachment.depthLoadOp = 'clear';
      depthStencilAttachment.depthStoreOp = 'store';
    }
    if (isStencilTextureFormat(format)) {
      depthStencilAttachment.stencilLoadOp = 'load';
      depthStencilAttachment.stencilStoreOp = 'store';
    }
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: outputTexture.createView(),
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }],

      depthStencilAttachment
    });

    for (let stencilBitIndex = 0; stencilBitIndex < stencilBitCount; ++stencilBitIndex) {
      const renderPipelineDescriptor = renderPipelineDescriptorBase;
      assert(renderPipelineDescriptor.depthStencil !== undefined);
      renderPipelineDescriptor.depthStencil.stencilReadMask = 1 << stencilBitIndex;
      const renderPipeline = t.device.createRenderPipeline(renderPipelineDescriptor);

      renderPass.setPipeline(renderPipeline);
      renderPass.setStencilReference(1 << stencilBitIndex);
      renderPass.setBindGroup(0, bindGroup, [stencilBitIndex * kMinDynamicBufferOffsetAlignment]);
      renderPass.draw(6);
    }
    renderPass.end();

    // Check outputTexture by copying the content of outputTexture into outputStagingBuffer and
    // checking all the data in outputStagingBuffer.
    const outputStagingBuffer = t.createBufferTracked({
      size: copyFromOutputTextureLayout.byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });
    encoder.copyTextureToBuffer(
      {
        texture: outputTexture
      },
      {
        buffer: outputStagingBuffer,
        bytesPerRow: copyFromOutputTextureLayout.bytesPerRow,
        rowsPerImage: copyFromOutputTextureLayout.rowsPerImage
      },
      outputTextureSize
    );

    t.queue.submit([encoder.finish()]);

    // Check the valid data in outputStagingBuffer once per row.
    for (let y = 0; y < copyFromOutputTextureLayout.mipSize[1]; ++y) {
      const dataStart =
      initialDataOffset + bytesPerRow * rowsPerImage * stencilTextureLayer + bytesPerRow * y;
      t.expectGPUBufferValuesEqual(
        outputStagingBuffer,
        initialData.slice(dataStart, dataStart + copyFromOutputTextureLayout.mipSize[0]),
        copyFromOutputTextureLayout.bytesPerRow * y
      );
    }
  }
});