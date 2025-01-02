/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test related to depth buffer, depth op, compare func, etc.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';

import { kDepthStencilFormats, kTextureFormatInfo } from '../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { TexelView } from '../../../util/texture/texel_view.js';

const backgroundColor = [0x00, 0x00, 0x00, 0xff];
const triangleColor = [0xff, 0xff, 0xff, 0xff];

const kBaseColor = new Float32Array([1.0, 1.0, 1.0, 1.0]);
const kRedStencilColor = new Float32Array([1.0, 0.0, 0.0, 1.0]);
const kGreenStencilColor = new Float32Array([0.0, 1.0, 0.0, 1.0]);







class DepthTest extends TextureTestMixin(GPUTest) {
  runDepthStateTest(testStates, expectedColor) {
    const renderTargetFormat = 'rgba8unorm';

    const renderTarget = this.createTextureTracked({
      format: renderTargetFormat,
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });

    const depthStencilFormat = 'depth24plus-stencil8';
    const depthTexture = this.createTextureTracked({
      size: { width: 1, height: 1, depthOrArrayLayers: 1 },
      format: depthStencilFormat,
      sampleCount: 1,
      mipLevelCount: 1,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_DST
    });

    const depthStencilAttachment = {
      view: depthTexture.createView(),
      depthLoadOp: 'load',
      depthStoreOp: 'store',
      stencilLoadOp: 'load',
      stencilStoreOp: 'store'
    };

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: renderTarget.createView(),
        loadOp: 'load',
        storeOp: 'store'
      }],

      depthStencilAttachment
    });

    // Draw a triangle with the given depth state, color, and depth.
    for (const test of testStates) {
      const testPipeline = this.createRenderPipelineForTest(test.state, test.depth);
      pass.setPipeline(testPipeline);
      pass.setBindGroup(
        0,
        this.createBindGroupForTest(testPipeline.getBindGroupLayout(0), test.color)
      );
      pass.draw(1);
    }

    pass.end();
    this.device.queue.submit([encoder.finish()]);

    const expColor = {
      R: expectedColor[0],
      G: expectedColor[1],
      B: expectedColor[2],
      A: expectedColor[3]
    };
    const expTexelView = TexelView.fromTexelsAsColors(renderTargetFormat, (_coords) => expColor);

    this.expectTexelViewComparisonIsOkInTexture({ texture: renderTarget }, expTexelView, [1, 1]);
  }

  createRenderPipelineForTest(
  depthStencil,
  depth)
  {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: `
            @vertex
            fn main(@builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
                return vec4<f32>(0.0, 0.0, ${depth}, 1.0);
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

export const g = makeTestGroup(DepthTest);

g.test('depth_disabled').
desc('Tests render results with depth test disabled.').
fn((t) => {
  const depthSpencilFormat = 'depth24plus-stencil8';
  const state = {
    format: depthSpencilFormat,
    depthWriteEnabled: false,
    depthCompare: 'always'
  };

  const testStates = [
  { state, color: kBaseColor, depth: 0.0 },
  { state, color: kRedStencilColor, depth: 0.5 },
  { state, color: kGreenStencilColor, depth: 1.0 }];


  // Test that for all combinations and ensure the last triangle drawn is the one visible
  // regardless of depth testing.
  for (let last = 0; last < 3; ++last) {
    const i = (last + 1) % 3;
    const j = (last + 2) % 3;

    t.runDepthStateTest([testStates[i], testStates[j], testStates[last]], testStates[last].color);
    t.runDepthStateTest([testStates[j], testStates[i], testStates[last]], testStates[last].color);
  }
});

g.test('depth_write_disabled').
desc(
  `
  Test that depthWriteEnabled behaves as expected.
  If enabled, a depth value of 0.0 is written.
  If disabled, it's not written, so it keeps the previous value of 1.0.
  Use a depthCompare: 'equal' check at the end to check the value.
  `
).
params((u) =>
u //
.combineWithParams([
{ depthWriteEnabled: false, lastDepth: 0.0, _expectedColor: kRedStencilColor },
{ depthWriteEnabled: true, lastDepth: 0.0, _expectedColor: kGreenStencilColor },
{ depthWriteEnabled: false, lastDepth: 1.0, _expectedColor: kGreenStencilColor },
{ depthWriteEnabled: true, lastDepth: 1.0, _expectedColor: kRedStencilColor }]
)
).
fn((t) => {
  const { depthWriteEnabled, lastDepth, _expectedColor } = t.params;

  const depthSpencilFormat = 'depth24plus-stencil8';

  const stencilState = {
    compare: 'always',
    failOp: 'keep',
    depthFailOp: 'keep',
    passOp: 'keep'
  };

  const baseState = {
    format: depthSpencilFormat,
    depthWriteEnabled: true,
    depthCompare: 'always',
    stencilFront: stencilState,
    stencilBack: stencilState,
    stencilReadMask: 0xff,
    stencilWriteMask: 0xff
  };

  const depthWriteState = {
    format: depthSpencilFormat,
    depthWriteEnabled,
    depthCompare: 'always',
    stencilFront: stencilState,
    stencilBack: stencilState,
    stencilReadMask: 0xff,
    stencilWriteMask: 0xff
  };

  const checkState = {
    format: depthSpencilFormat,
    depthWriteEnabled: false,
    depthCompare: 'equal',
    stencilFront: stencilState,
    stencilBack: stencilState,
    stencilReadMask: 0xff,
    stencilWriteMask: 0xff
  };

  const testStates = [
  // Draw a base point with depth write enabled.
  { state: baseState, color: kBaseColor, depth: 1.0 },
  // Draw a second point without depth write enabled.
  { state: depthWriteState, color: kRedStencilColor, depth: 0.0 },
  // Draw a third point which should occlude the second even though it is behind it.
  { state: checkState, color: kGreenStencilColor, depth: lastDepth }];


  t.runDepthStateTest(testStates, _expectedColor);
});

g.test('depth_test_fail').
desc(
  `
  Test that render results on depth test failure cases with 'less' depthCompare operation and
  depthWriteEnabled is true.
  `
).
params((u) =>
u //
.combineWithParams([
{ secondDepth: 1.0, lastDepth: 2.0, _expectedColor: kBaseColor }, // fail -> fail.
{ secondDepth: 0.0, lastDepth: 2.0, _expectedColor: kRedStencilColor }, // pass -> fail.
{ secondDepth: 2.0, lastDepth: 0.9, _expectedColor: kGreenStencilColor } // fail -> pass.
])
).
fn((t) => {
  const { secondDepth, lastDepth, _expectedColor } = t.params;

  const depthSpencilFormat = 'depth24plus-stencil8';

  const baseState = {
    format: depthSpencilFormat,
    depthWriteEnabled: true,
    depthCompare: 'always',
    stencilReadMask: 0xff,
    stencilWriteMask: 0xff
  };

  const depthTestState = {
    format: depthSpencilFormat,
    depthWriteEnabled: true,
    depthCompare: 'less',
    stencilReadMask: 0xff,
    stencilWriteMask: 0xff
  };

  const testStates = [
  { state: baseState, color: kBaseColor, depth: 1.0 },
  { state: depthTestState, color: kRedStencilColor, depth: secondDepth },
  { state: depthTestState, color: kGreenStencilColor, depth: lastDepth }];


  t.runDepthStateTest(testStates, _expectedColor);
});

// Use a depth value that's not exactly 0.5 because it is exactly between two depth16unorm value and
// can get rounded either way (and a different way between shaders and clearDepthValue).
const kMiddleDepthValue = 0.5001;

g.test('depth_compare_func').
desc(
  `Tests each depth compare function works properly. Clears the depth attachment to various values, and renders a point at depth 0.5 with various depthCompare modes.`
).
params((u) =>
u.
combine(
  'format',
  kDepthStencilFormats.filter((format) => kTextureFormatInfo[format].depth)
).
combineWithParams([
{ depthCompare: 'never', depthClearValue: 1.0, _expected: backgroundColor },
{ depthCompare: 'never', depthClearValue: kMiddleDepthValue, _expected: backgroundColor },
{ depthCompare: 'never', depthClearValue: 0.0, _expected: backgroundColor },
{ depthCompare: 'less', depthClearValue: 1.0, _expected: triangleColor },
{ depthCompare: 'less', depthClearValue: kMiddleDepthValue, _expected: backgroundColor },
{ depthCompare: 'less', depthClearValue: 0.0, _expected: backgroundColor },
{ depthCompare: 'less-equal', depthClearValue: 1.0, _expected: triangleColor },
{
  depthCompare: 'less-equal',
  depthClearValue: kMiddleDepthValue,
  _expected: triangleColor
},
{ depthCompare: 'less-equal', depthClearValue: 0.0, _expected: backgroundColor },
{ depthCompare: 'equal', depthClearValue: 1.0, _expected: backgroundColor },
{ depthCompare: 'equal', depthClearValue: kMiddleDepthValue, _expected: triangleColor },
{ depthCompare: 'equal', depthClearValue: 0.0, _expected: backgroundColor },
{ depthCompare: 'not-equal', depthClearValue: 1.0, _expected: triangleColor },
{
  depthCompare: 'not-equal',
  depthClearValue: kMiddleDepthValue,
  _expected: backgroundColor
},
{ depthCompare: 'not-equal', depthClearValue: 0.0, _expected: triangleColor },
{ depthCompare: 'greater-equal', depthClearValue: 1.0, _expected: backgroundColor },
{
  depthCompare: 'greater-equal',
  depthClearValue: kMiddleDepthValue,
  _expected: triangleColor
},
{ depthCompare: 'greater-equal', depthClearValue: 0.0, _expected: triangleColor },
{ depthCompare: 'greater', depthClearValue: 1.0, _expected: backgroundColor },
{ depthCompare: 'greater', depthClearValue: kMiddleDepthValue, _expected: backgroundColor },
{ depthCompare: 'greater', depthClearValue: 0.0, _expected: triangleColor },
{ depthCompare: 'always', depthClearValue: 1.0, _expected: triangleColor },
{ depthCompare: 'always', depthClearValue: kMiddleDepthValue, _expected: triangleColor },
{ depthCompare: 'always', depthClearValue: 0.0, _expected: triangleColor }]
)
).
beforeAllSubcases((t) => {
  t.selectDeviceForTextureFormatOrSkipTestCase(t.params.format);
}).
fn((t) => {
  const { depthCompare, depthClearValue, _expected, format } = t.params;

  const colorAttachmentFormat = 'rgba8unorm';
  const colorAttachment = t.createTextureTracked({
    format: colorAttachmentFormat,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });
  const colorAttachmentView = colorAttachment.createView();

  const depthTexture = t.createTextureTracked({
    size: { width: 1, height: 1 },
    format,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING
  });
  const depthTextureView = depthTexture.createView();

  const pipelineDescriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
            @vertex fn main(
              @builtin(vertex_index) VertexIndex : u32) -> @builtin(position) vec4<f32> {
              return vec4<f32>(0.5, 0.5, ${kMiddleDepthValue}, 1.0);
            }
            `
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(1.0, 1.0, 1.0, 1.0);
            }
            `
      }),
      entryPoint: 'main',
      targets: [{ format: colorAttachmentFormat }]
    },
    primitive: { topology: 'point-list' },
    depthStencil: {
      depthWriteEnabled: true,
      depthCompare,
      format
    }
  };
  const pipeline = t.device.createRenderPipeline(pipelineDescriptor);

  const encoder = t.device.createCommandEncoder();
  const depthStencilAttachment = {
    view: depthTextureView,
    depthClearValue,
    depthLoadOp: 'clear',
    depthStoreOp: 'store'
  };
  if (kTextureFormatInfo[format].stencil) {
    depthStencilAttachment.stencilClearValue = 0;
    depthStencilAttachment.stencilLoadOp = 'clear';
    depthStencilAttachment.stencilStoreOp = 'store';
  }
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachmentView,
      clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }],

    depthStencilAttachment
  });
  pass.setPipeline(pipeline);
  pass.draw(1);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, [
  {
    coord: { x: 0, y: 0 },
    exp: new Uint8Array(_expected)
  }]
  );
});

g.test('reverse_depth').
desc(
  `Tests simple rendering with reversed depth buffer, ensures depth test works properly: fragments are in correct order and out of range fragments are clipped.
    Note that in real use case the depth range remapping is done by the modified projection matrix.
(see https://developer.nvidia.com/content/depth-precision-visualized).`
).
params((u) => u.combine('reversed', [false, true])).
fn((t) => {
  const colorAttachmentFormat = 'rgba8unorm';
  const colorAttachment = t.createTextureTracked({
    format: colorAttachmentFormat,
    size: { width: 1, height: 1, depthOrArrayLayers: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
  });
  const colorAttachmentView = colorAttachment.createView();

  const depthBufferFormat = 'depth32float';
  const depthTexture = t.createTextureTracked({
    size: { width: 1, height: 1 },
    format: depthBufferFormat,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING
  });
  const depthTextureView = depthTexture.createView();

  const pipelineDescriptor = {
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `
            struct Output {
              @builtin(position) Position : vec4<f32>,
              @location(0) color : vec4<f32>,
            };

            @vertex fn main(
              @builtin(vertex_index) VertexIndex : u32,
              @builtin(instance_index) InstanceIndex : u32) -> Output {
              let zv = array(0.2, 0.3, -0.1, 1.1);
              let z = zv[InstanceIndex];

              var output : Output;
              output.Position = vec4<f32>(0.5, 0.5, z, 1.0);
              var colors : array<vec4<f32>, 4> = array<vec4<f32>, 4>(
                  vec4<f32>(1.0, 0.0, 0.0, 1.0),
                  vec4<f32>(0.0, 1.0, 0.0, 1.0),
                  vec4<f32>(0.0, 0.0, 1.0, 1.0),
                  vec4<f32>(1.0, 1.0, 1.0, 1.0)
              );
              output.color = colors[InstanceIndex];
              return output;
            }
            `
      }),
      entryPoint: 'main'
    },
    fragment: {
      module: t.device.createShaderModule({
        code: `
            @fragment fn main(
              @location(0) color : vec4<f32>
              ) -> @location(0) vec4<f32> {
              return color;
            }
            `
      }),
      entryPoint: 'main',
      targets: [{ format: colorAttachmentFormat }]
    },
    primitive: { topology: 'point-list' },
    depthStencil: {
      depthWriteEnabled: true,
      depthCompare: t.params.reversed ? 'greater' : 'less',
      format: depthBufferFormat
    }
  };
  const pipeline = t.device.createRenderPipeline(pipelineDescriptor);

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachmentView,
      clearValue: { r: 0.5, g: 0.5, b: 0.5, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    }],

    depthStencilAttachment: {
      view: depthTextureView,

      depthClearValue: t.params.reversed ? 0.0 : 1.0,
      depthLoadOp: 'clear',
      depthStoreOp: 'store'
    }
  });
  pass.setPipeline(pipeline);
  pass.draw(1, 4);
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  t.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, [
  {
    coord: { x: 0, y: 0 },
    exp: new Uint8Array(
      t.params.reversed ? [0x00, 0xff, 0x00, 0xff] : [0xff, 0x00, 0x00, 0xff]
    )
  }]
  );
});