/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests render results with different depth bias values like 'positive', 'negative',
'slope', 'clamp', etc.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable } from '../../../../common/util/util.js';
import {
  kTextureFormatInfo } from


'../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { TexelView } from '../../../util/texture/texel_view.js';var

QuadAngle = /*#__PURE__*/function (QuadAngle) {QuadAngle[QuadAngle["Flat"] = 0] = "Flat";QuadAngle[QuadAngle["TiltedX"] = 1] = "TiltedX";return QuadAngle;}(QuadAngle || {});




// Floating point depth buffers use the following formula to calculate bias
// bias = depthBias * 2 ** (exponent(max z of primitive) - number of bits in mantissa) +
//        slopeScale * maxSlope
// https://docs.microsoft.com/en-us/windows/win32/direct3d11/d3d10-graphics-programming-guide-output-merger-stage-depth-bias
// https://www.khronos.org/registry/vulkan/specs/1.2-extensions/man/html/vkCmdSetDepthBias.html
// https://developer.apple.com/documentation/metal/mtlrendercommandencoder/1516269-setdepthbias
//
// To get a final bias of 0.25 for primitives with z = 0.25, we can use
// depthBias = 0.25 / (2 ** (-2 - 23)) = 8388608.
const kPointTwoFiveBiasForPointTwoFiveZOnFloat = 8388608;

class DepthBiasTest extends TextureTestMixin(GPUTest) {
  runDepthBiasTestInternal(
  depthFormat,
  {
    quadAngle,
    bias,
    biasSlopeScale,
    biasClamp,
    initialDepth






  })
  {
    const renderTargetFormat = 'rgba8unorm';
    const depthFormatInfo = kTextureFormatInfo[depthFormat];

    let vertexShaderCode;
    switch (quadAngle) {
      case QuadAngle.Flat:
        // Draw a square at z = 0.25.
        vertexShaderCode = `
          @vertex
          fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
            var pos = array<vec2<f32>, 6>(
            vec2<f32>(-1.0, -1.0),
            vec2<f32>( 1.0, -1.0),
            vec2<f32>(-1.0,  1.0),
            vec2<f32>(-1.0,  1.0),
            vec2<f32>( 1.0, -1.0),
            vec2<f32>( 1.0,  1.0));
            return vec4<f32>(pos[VertexIndex], 0.25, 1.0);
          }
          `;
        break;
      case QuadAngle.TiltedX:
        // Draw a square ranging from 0 to 0.5, bottom to top.
        vertexShaderCode = `
          @vertex
          fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
            var pos = array<vec3<f32>, 6>(
            vec3<f32>(-1.0, -1.0, 0.0),
            vec3<f32>( 1.0, -1.0, 0.0),
            vec3<f32>(-1.0,  1.0, 0.5),
            vec3<f32>(-1.0,  1.0, 0.5),
            vec3<f32>( 1.0, -1.0, 0.0),
            vec3<f32>( 1.0,  1.0, 0.5));
            return vec4<f32>(pos[VertexIndex], 1.0);
          }
          `;
        break;
      default:
        unreachable();
    }

    const renderTarget = this.trackForCleanup(
      this.device.createTexture({
        format: renderTargetFormat,
        size: { width: 1, height: 1, depthOrArrayLayers: 1 },
        usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
      })
    );

    const depthTexture = this.trackForCleanup(
      this.device.createTexture({
        size: { width: 1, height: 1, depthOrArrayLayers: 1 },
        format: depthFormat,
        sampleCount: 1,
        mipLevelCount: 1,
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
      })
    );

    const depthStencilAttachment = {
      view: depthTexture.createView(),
      depthLoadOp: depthFormatInfo.depth ? 'clear' : undefined,
      depthStoreOp: depthFormatInfo.depth ? 'store' : undefined,
      stencilLoadOp: depthFormatInfo.stencil ? 'clear' : undefined,
      stencilStoreOp: depthFormatInfo.stencil ? 'store' : undefined,
      depthClearValue: initialDepth
    };

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: renderTarget.createView(),
        storeOp: 'store',
        loadOp: 'load'
      }],

      depthStencilAttachment
    });

    let depthCompare = 'always';
    if (depthFormat !== 'depth32float') {
      depthCompare = 'greater';
    }

    const testState = {
      format: depthFormat,
      depthCompare,
      depthWriteEnabled: true,
      depthBias: bias,
      depthBiasSlopeScale: biasSlopeScale,
      depthBiasClamp: biasClamp
    };

    // Draw a square with the given depth state and bias values.
    const testPipeline = this.createRenderPipelineForTest(vertexShaderCode, testState);
    pass.setPipeline(testPipeline);
    pass.draw(6);
    pass.end();
    this.device.queue.submit([encoder.finish()]);

    return { renderTarget, depthTexture };
  }

  runDepthBiasTest(
  depthFormat,
  {
    quadAngle,
    bias,
    biasSlopeScale,
    biasClamp,
    _expectedDepth






  })
  {
    const { depthTexture } = this.runDepthBiasTestInternal(depthFormat, {
      quadAngle,
      bias,
      biasSlopeScale,
      biasClamp,
      initialDepth: 0
    });

    const expColor = { Depth: _expectedDepth };
    const expTexelView = TexelView.fromTexelsAsColors(depthFormat, (_coords) => expColor);
    this.expectTexelViewComparisonIsOkInTexture({ texture: depthTexture }, expTexelView, [1, 1]);
  }

  runDepthBiasTestFor24BitFormat(
  depthFormat,
  {
    quadAngle,
    bias,
    biasSlopeScale,
    biasClamp,
    _expectedColor






  })
  {
    const { renderTarget } = this.runDepthBiasTestInternal(depthFormat, {
      quadAngle,
      bias,
      biasSlopeScale,
      biasClamp,
      initialDepth: 0.4
    });

    const renderTargetFormat = 'rgba8unorm';
    const expColor = {
      R: _expectedColor[0],
      G: _expectedColor[1],
      B: _expectedColor[2],
      A: _expectedColor[3]
    };
    const expTexelView = TexelView.fromTexelsAsColors(renderTargetFormat, (_coords) => expColor);
    this.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [1, 1]);
  }

  createRenderPipelineForTest(
  vertex,
  depthStencil)
  {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: vertex
        }),
        entryPoint: 'main'
      },
      fragment: {
        targets: [{ format: 'rgba8unorm' }],
        module: this.device.createShaderModule({
          code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(1.0, 0.0, 0.0, 1.0);
            }`
        }),
        entryPoint: 'main'
      },
      depthStencil
    });
  }
}

export const g = makeTestGroup(DepthBiasTest);

g.test('depth_bias').
desc(
  `
  Tests that a square with different depth bias values like 'positive', 'negative',
  'slope', 'clamp', etc. is drawn as expected.
  `
).
params((u) =>
u //
.combineWithParams([
{
  quadAngle: QuadAngle.Flat,
  bias: kPointTwoFiveBiasForPointTwoFiveZOnFloat,
  biasSlopeScale: 0,
  biasClamp: 0,
  _expectedDepth: 0.5
},
{
  quadAngle: QuadAngle.Flat,
  bias: kPointTwoFiveBiasForPointTwoFiveZOnFloat,
  biasSlopeScale: 0,
  biasClamp: 0.125,
  _expectedDepth: 0.375
},
{
  quadAngle: QuadAngle.Flat,
  bias: -kPointTwoFiveBiasForPointTwoFiveZOnFloat,
  biasSlopeScale: 0,
  biasClamp: 0.125,
  _expectedDepth: 0
},
{
  quadAngle: QuadAngle.Flat,
  bias: -kPointTwoFiveBiasForPointTwoFiveZOnFloat,
  biasSlopeScale: 0,
  biasClamp: -0.125,
  _expectedDepth: 0.125
},
{
  quadAngle: QuadAngle.TiltedX,
  bias: 0,
  biasSlopeScale: 0,
  biasClamp: 0,
  _expectedDepth: 0.25
},
{
  quadAngle: QuadAngle.TiltedX,
  bias: 0,
  biasSlopeScale: 1,
  biasClamp: 0,
  _expectedDepth: 0.75
},
{
  quadAngle: QuadAngle.TiltedX,
  bias: 0,
  biasSlopeScale: -0.5,
  biasClamp: 0,
  _expectedDepth: 0
}]
)
).
fn((t) => {
  t.runDepthBiasTest('depth32float', t.params);
});

g.test('depth_bias_24bit_format').
desc(
  `
  Tests that a square with different depth bias values like 'positive', 'negative',
  'slope', 'clamp', etc. is drawn as expected with 24 bit depth format.

  TODO: Enhance these tests by reading back the depth (emulating the copy using texture sampling)
  and checking the result directly, like the non-24-bit depth tests, instead of just relying on
  whether the depth test passes or fails.
  `
).
params((u) =>
u //
.combine('format', ['depth24plus', 'depth24plus-stencil8']).
combineWithParams([
{
  quadAngle: QuadAngle.Flat,
  bias: 0.25 * (1 << 25),
  biasSlopeScale: 0,
  biasClamp: 0,
  _expectedColor: new Float32Array([1.0, 0.0, 0.0, 1.0])
},
{
  quadAngle: QuadAngle.TiltedX,
  bias: 0.25 * (1 << 25),
  biasSlopeScale: 1,
  biasClamp: 0,
  _expectedColor: new Float32Array([1.0, 0.0, 0.0, 1.0])
},
{
  quadAngle: QuadAngle.Flat,
  bias: 0.25 * (1 << 25),
  biasSlopeScale: 0,
  biasClamp: 0.1,
  _expectedColor: new Float32Array([0.0, 0.0, 0.0, 0.0])
}]
)
).
fn((t) => {
  const { format } = t.params;
  t.runDepthBiasTestFor24BitFormat(format, t.params);
});