/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that the final sample mask is the logical AND of all the relevant masks, including
the rasterization mask, sample mask, fragment output mask, and alpha to coverage mask (when alphaToCoverageEnabled === true).

Also tested:
- The positions of samples in the standard sample patterns.
- Per-sample interpolation sampling: @interpolate(perspective, sample).

TODO: add a test without a 0th color attachment (sparse color attachment), with different color attachments and alpha value output.
The cross-platform behavior is unknown. could be any of:
- coverage is always 100%
- coverage is always 0%
- it uses the first non-null attachment
- it's an error
Details could be found at: https://github.com/gpuweb/cts/issues/2201
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert, range } from '../../../../common/util/util.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { checkElementsPassPredicate, checkElementsEqual } from '../../../util/check_contents.js';
import { TypeF32, TypeU32 } from '../../../util/conversion.js';
import { TexelView } from '../../../util/texture/texel_view.js';

const kColors = [
// Red
new Uint8Array([0xff, 0, 0, 0xff]),
// Green
new Uint8Array([0, 0xff, 0, 0xff]),
// Blue
new Uint8Array([0, 0, 0xff, 0xff]),
// Yellow
new Uint8Array([0xff, 0xff, 0, 0xff])];


const kDepthClearValue = 1.0;
const kDepthWriteValue = 0.0;
const kStencilClearValue = 0;
const kStencilReferenceValue = 0xff;

// Format of the render target and resolve target
const format = 'rgba8unorm';

// Format of depth stencil attachment
const depthStencilFormat = 'depth24plus-stencil8';

const kRenderTargetSize = 1;

function hasSample(
rasterizationMask,
sampleMask,
fragmentShaderOutputMask,
sampleIndex = 0)
{
  return (rasterizationMask & sampleMask & fragmentShaderOutputMask & 1 << sampleIndex) > 0;
}

function getExpectedColorData(
sampleCount,
rasterizationMask,
sampleMask,
fragmentShaderOutputMaskOrAlphaToCoverageMask)
{
  const expectedData = new Float32Array(sampleCount * 4);
  if (sampleCount === 1) {
    if (hasSample(rasterizationMask, sampleMask, fragmentShaderOutputMaskOrAlphaToCoverageMask)) {
      // Texel 3 is sampled at the pixel center
      expectedData[0] = kColors[3][0] / 0xff;
      expectedData[1] = kColors[3][1] / 0xff;
      expectedData[2] = kColors[3][2] / 0xff;
      expectedData[3] = kColors[3][3] / 0xff;
    }
  } else {
    for (let i = 0; i < sampleCount; i++) {
      if (
      hasSample(rasterizationMask, sampleMask, fragmentShaderOutputMaskOrAlphaToCoverageMask, i))
      {
        const o = i * 4;
        expectedData[o + 0] = kColors[i][0] / 0xff;
        expectedData[o + 1] = kColors[i][1] / 0xff;
        expectedData[o + 2] = kColors[i][2] / 0xff;
        expectedData[o + 3] = kColors[i][3] / 0xff;
      }
    }
  }
  return expectedData;
}

function getExpectedDepthData(
sampleCount,
rasterizationMask,
sampleMask,
fragmentShaderOutputMaskOrAlphaToCoverageMask)
{
  const expectedData = new Float32Array(sampleCount);
  for (let i = 0; i < sampleCount; i++) {
    const s = hasSample(
      rasterizationMask,
      sampleMask,
      fragmentShaderOutputMaskOrAlphaToCoverageMask,
      i
    );
    expectedData[i] = s ? kDepthWriteValue : kDepthClearValue;
  }
  return expectedData;
}

function getExpectedStencilData(
sampleCount,
rasterizationMask,
sampleMask,
fragmentShaderOutputMaskOrAlphaToCoverageMask)
{
  const expectedData = new Uint32Array(sampleCount);
  for (let i = 0; i < sampleCount; i++) {
    const s = hasSample(
      rasterizationMask,
      sampleMask,
      fragmentShaderOutputMaskOrAlphaToCoverageMask,
      i
    );
    expectedData[i] = s ? kStencilReferenceValue : kStencilClearValue;
  }
  return expectedData;
}

const kSampleMaskTestShader = `
struct Varyings {
  @builtin(position) Position : vec4<f32>,
  @location(0) @interpolate(flat) uvFlat : vec2<f32>,
  @location(1) @interpolate(perspective, sample) uvInterpolated : vec2<f32>,
}

//
// Vertex shader
//

@vertex
fn vmain(@builtin(vertex_index) VertexIndex : u32,
    @builtin(instance_index) InstanceIndex : u32) -> Varyings {
  // Standard sample locations within a pixel, where the pixel ranges from (-1,-1) to (1,1), and is
  // centered at (0,0) (NDC - the test uses a 1x1 render target).
  // https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_standard_multisample_quality_levels
  var sampleCenters = array(
      // sampleCount = 1
      vec2f(0, 0),
      // sampleCount = 4
      vec2f(-2,  6) / 8,
      vec2f( 6,  2) / 8,
      vec2f(-6, -2) / 8,
      vec2f( 2, -6) / 8,
    );
  // A tiny quad to draw around the sample center to ensure we hit only the expected point.
  let kTinyQuadRadius = 1.0 / 32;
  var tinyQuad = array(
    vec2f( kTinyQuadRadius,  kTinyQuadRadius),
    vec2f( kTinyQuadRadius, -kTinyQuadRadius),
    vec2f(-kTinyQuadRadius, -kTinyQuadRadius),
    vec2f( kTinyQuadRadius,  kTinyQuadRadius),
    vec2f(-kTinyQuadRadius, -kTinyQuadRadius),
    vec2f(-kTinyQuadRadius,  kTinyQuadRadius),
    );

  var uvsFlat = array(
      // sampleCount = 1
      // Note: avoids hitting the point between the 4 texels.
      vec2f(0.51, 0.51),
      // sampleCount = 4
      vec2f(0.25, 0.25),
      vec2f(0.75, 0.25),
      vec2f(0.25, 0.75),
      vec2f(0.75, 0.75),
    );
  var uvsInterpolated = array(
      // center quad
      // Note: the interpolated point will be exactly in the middle of the 4 texels.
      // The test expects to get texel 1,1 (the 3rd texel) in this case.
      vec2f(1.0, 0.0),
      vec2f(1.0, 1.0),
      vec2f(0.0, 1.0),
      vec2f(1.0, 0.0),
      vec2f(0.0, 1.0),
      vec2f(0.0, 0.0),

      // top-left quad (texel 0)
      vec2f(0.5, 0.0),
      vec2f(0.5, 0.5),
      vec2f(0.0, 0.5),
      vec2f(0.5, 0.0),
      vec2f(0.0, 0.5),
      vec2f(0.0, 0.0),

      // top-right quad (texel 1)
      vec2f(1.0, 0.0),
      vec2f(1.0, 0.5),
      vec2f(0.5, 0.5),
      vec2f(1.0, 0.0),
      vec2f(0.5, 0.5),
      vec2f(0.5, 0.0),

      // bottom-left quad (texel 2)
      vec2f(0.5, 0.5),
      vec2f(0.5, 1.0),
      vec2f(0.0, 1.0),
      vec2f(0.5, 0.5),
      vec2f(0.0, 1.0),
      vec2f(0.0, 0.5),

      // bottom-right quad (texel 3)
      vec2f(1.0, 0.5),
      vec2f(1.0, 1.0),
      vec2f(0.5, 1.0),
      vec2f(1.0, 0.5),
      vec2f(0.5, 1.0),
      vec2f(0.5, 0.5)
    );

  var output : Varyings;
  let pos = sampleCenters[InstanceIndex] + tinyQuad[VertexIndex];
  output.Position = vec4(pos, ${kDepthWriteValue}, 1.0);
  output.uvFlat = uvsFlat[InstanceIndex];
  output.uvInterpolated = uvsInterpolated[InstanceIndex * 6 + VertexIndex];
  return output;
}

//
// Fragment shaders
//

@group(0) @binding(0) var mySampler: sampler;
@group(0) @binding(1) var myTexture: texture_2d<f32>;

// For test named 'fragment_output_mask'

@group(0) @binding(2) var<uniform> fragMask: u32;
struct FragmentOutput1 {
  @builtin(sample_mask) mask : u32,
  @location(0) color : vec4<f32>,
}
@fragment fn fmain__fragment_output_mask__flat(varyings: Varyings) -> FragmentOutput1 {
  return FragmentOutput1(fragMask, textureSample(myTexture, mySampler, varyings.uvFlat));
}
@fragment fn fmain__fragment_output_mask__interp(varyings: Varyings) -> FragmentOutput1 {
  return FragmentOutput1(fragMask, textureSample(myTexture, mySampler, varyings.uvInterpolated));
}

// For test named 'alpha_to_coverage_mask'

struct FragmentOutput2 {
  @location(0) color0 : vec4<f32>,
  @location(1) color1 : vec4<f32>,
}
@group(0) @binding(2) var<uniform> alpha: vec2<f32>;
@fragment fn fmain__alpha_to_coverage_mask__flat(varyings: Varyings) -> FragmentOutput2 {
  var c = textureSample(myTexture, mySampler, varyings.uvFlat);
  return FragmentOutput2(vec4(c.xyz, alpha[0]), vec4(c.xyz, alpha[1]));
}
@fragment fn fmain__alpha_to_coverage_mask__interp(varyings: Varyings) -> FragmentOutput2 {
  var c = textureSample(myTexture, mySampler, varyings.uvInterpolated);
  return FragmentOutput2(vec4(c.xyz, alpha[0]), vec4(c.xyz, alpha[1]));
}
`;

class F extends TextureTestMixin(GPUTest) {



  async init() {
    await super.init();
    if (this.isCompatibility) {
      this.skip('WGSL sample_mask is not supported in compatibility mode');
    }
    // Create a 2x2 color texture to sample from
    // texel 0 - Red
    // texel 1 - Green
    // texel 2 - Blue
    // texel 3 - Yellow
    const kSampleTextureSize = 2;
    this.sampleTexture = this.createTextureFromTexelView(
      TexelView.fromTexelsAsBytes(format, (coord) => {
        const id = coord.x + coord.y * kSampleTextureSize;
        return kColors[id];
      }),
      {
        size: [kSampleTextureSize, kSampleTextureSize, 1],
        usage:
        GPUTextureUsage.TEXTURE_BINDING |
        GPUTextureUsage.COPY_DST |
        GPUTextureUsage.RENDER_ATTACHMENT
      }
    );

    this.sampler = this.device.createSampler({
      magFilter: 'nearest',
      minFilter: 'nearest'
    });
  }

  GetTargetTexture(
  sampleCount,
  rasterizationMask,
  pipeline,
  uniformBuffer,
  colorTargetsCount = 1)
  {
    assert(this.sampleTexture !== undefined);
    assert(this.sampler !== undefined);

    const uniformBindGroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      {
        binding: 0,
        resource: this.sampler
      },
      {
        binding: 1,
        resource: this.sampleTexture.createView()
      },
      {
        binding: 2,
        resource: {
          buffer: uniformBuffer
        }
      }]

    });

    const renderTargetTextures = [];
    const resolveTargetTextures = [];
    for (let i = 0; i < colorTargetsCount; i++) {
      const renderTargetTexture = this.device.createTexture({
        format,
        size: {
          width: kRenderTargetSize,
          height: kRenderTargetSize,
          depthOrArrayLayers: 1
        },
        sampleCount,
        mipLevelCount: 1,
        usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING
      });
      renderTargetTextures.push(renderTargetTexture);

      const resolveTargetTexture =
      sampleCount === 1 ?
      null :
      this.device.createTexture({
        format,
        size: {
          width: kRenderTargetSize,
          height: kRenderTargetSize,
          depthOrArrayLayers: 1
        },
        sampleCount: 1,
        mipLevelCount: 1,
        usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
      });
      resolveTargetTextures.push(resolveTargetTexture);
    }

    const depthStencilTexture = this.device.createTexture({
      size: {
        width: kRenderTargetSize,
        height: kRenderTargetSize
      },
      format: depthStencilFormat,
      sampleCount,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING
    });

    const renderPassDescriptor = {
      colorAttachments: renderTargetTextures.map((renderTargetTexture, index) => {
        return {
          view: renderTargetTexture.createView(),
          resolveTarget: resolveTargetTextures[index]?.createView(),
          clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 0.0 },
          loadOp: 'clear',
          storeOp: 'store'
        };
      }),
      depthStencilAttachment: {
        view: depthStencilTexture.createView(),
        depthClearValue: kDepthClearValue,
        depthLoadOp: 'clear',
        depthStoreOp: 'store',
        stencilClearValue: kStencilClearValue,
        stencilLoadOp: 'clear',
        stencilStoreOp: 'store'
      }
    };
    const commandEncoder = this.device.createCommandEncoder();
    const passEncoder = commandEncoder.beginRenderPass(renderPassDescriptor);
    passEncoder.setPipeline(pipeline);
    passEncoder.setBindGroup(0, uniformBindGroup);
    passEncoder.setStencilReference(kStencilReferenceValue);

    if (sampleCount === 1) {
      if ((rasterizationMask & 1) !== 0) {
        // draw center quad
        passEncoder.draw(6, 1, 0, 0);
      }
    } else {
      assert(sampleCount === 4);
      if ((rasterizationMask & 1) !== 0) {
        // draw top-left quad
        passEncoder.draw(6, 1, 0, 1);
      }
      if ((rasterizationMask & 2) !== 0) {
        // draw top-right quad
        passEncoder.draw(6, 1, 0, 2);
      }
      if ((rasterizationMask & 4) !== 0) {
        // draw bottom-left quad
        passEncoder.draw(6, 1, 0, 3);
      }
      if ((rasterizationMask & 8) !== 0) {
        // draw bottom-right quad
        passEncoder.draw(6, 1, 0, 4);
      }
    }
    passEncoder.end();
    this.device.queue.submit([commandEncoder.finish()]);

    return {
      color: renderTargetTextures[0],
      depthStencil: depthStencilTexture
    };
  }

  CheckColorAttachmentResult(
  texture,
  sampleCount,
  rasterizationMask,
  sampleMask,
  fragmentShaderOutputMask)
  {
    const buffer = this.copySinglePixelTextureToBufferUsingComputePass(
      TypeF32, // correspond to 'rgba8unorm' format
      4,
      texture.createView(),
      sampleCount
    );

    const expected = getExpectedColorData(
      sampleCount,
      rasterizationMask,
      sampleMask,
      fragmentShaderOutputMask
    );
    this.expectGPUBufferValuesEqual(buffer, expected);
  }

  CheckDepthStencilResult(
  aspect,
  depthStencilTexture,
  sampleCount,
  rasterizationMask,
  sampleMask,
  fragmentShaderOutputMask)
  {
    const buffer = this.copySinglePixelTextureToBufferUsingComputePass(
      // Use f32 as the scalar type for depth (depth24plus, depth32float)
      // Use u32 as the scalar type for stencil (stencil8)
      aspect === 'depth-only' ? TypeF32 : TypeU32,
      1,
      depthStencilTexture.createView({ aspect }),
      sampleCount
    );

    const expected =
    aspect === 'depth-only' ?
    getExpectedDepthData(sampleCount, rasterizationMask, sampleMask, fragmentShaderOutputMask) :
    getExpectedStencilData(
      sampleCount,
      rasterizationMask,
      sampleMask,
      fragmentShaderOutputMask
    );
    this.expectGPUBufferValuesEqual(buffer, expected);
  }
}

export const g = makeTestGroup(F);

g.test('fragment_output_mask').
desc(
  `
Tests that the final sample mask is the logical AND of all the relevant masks -- meaning that the samples
not included in the final mask are discarded on any attachments including
- color outputs
- depth tests
- stencil operations

The test draws 0/1/1+ textured quads of which each sample in the standard 4-sample pattern results in a different color:
- Sample 0, Texel 0, top-left: Red
- Sample 1, Texel 1, top-left: Green
- Sample 2, Texel 2, top-left: Blue
- Sample 3, Texel 3, top-left: Yellow

The test checks each sample value of the render target texture and depth stencil texture using a compute pass to
textureLoad each sample index from the texture and write to a storage buffer to compare with expected values.

- for sampleCount = { 1, 4 } and various combinations of:
    - rasterization mask = { 0, ..., 2 ** sampleCount - 1 }
    - sample mask = { 0, 0b0001, 0b0010, 0b0111, 0b1011, 0b1101, 0b1110, 0b1111, 0b11110 }
    - fragment shader output @builtin(sample_mask) = { 0, 0b0001, 0b0010, 0b0111, 0b1011, 0b1101, 0b1110, 0b1111, 0b11110 }
- [choosing 0b11110 because the 5th bit should be ignored]
`
).
params((u) =>
u.
combine('interpolated', [false, true]).
combine('sampleCount', [1, 4]).
expand('rasterizationMask', function* (p) {
  const maxMask = 2 ** p.sampleCount - 1;
  for (let i = 0; i <= maxMask; i++) {
    yield i;
  }
}).
beginSubcases().
combine('sampleMask', [
0, 0b0001, 0b0010, 0b0111, 0b1011, 0b1101, 0b1110, 0b1111, 0b11110]
).
combine('fragmentShaderOutputMask', [
0, 0b0001, 0b0010, 0b0111, 0b1011, 0b1101, 0b1110, 0b1111, 0b11110]
)
).
fn((t) => {
  const { sampleCount, rasterizationMask, sampleMask, fragmentShaderOutputMask } = t.params;

  const fragmentMaskUniformBuffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(fragmentMaskUniformBuffer);
  t.device.queue.writeBuffer(
    fragmentMaskUniformBuffer,
    0,
    new Uint32Array([fragmentShaderOutputMask])
  );

  const module = t.device.createShaderModule({ code: kSampleMaskTestShader });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module, entryPoint: 'vmain' },
    fragment: {
      module,
      entryPoint: `fmain__fragment_output_mask__${t.params.interpolated ? 'interp' : 'flat'}`,
      targets: [{ format }]
    },
    primitive: { topology: 'triangle-list' },
    multisample: {
      count: sampleCount,
      mask: sampleMask,
      alphaToCoverageEnabled: false
    },
    depthStencil: {
      format: depthStencilFormat,
      depthWriteEnabled: true,
      depthCompare: 'always',

      stencilFront: {
        compare: 'always',
        passOp: 'replace'
      },
      stencilBack: {
        compare: 'always',
        passOp: 'replace'
      }
    }
  });

  const { color, depthStencil } = t.GetTargetTexture(
    sampleCount,
    rasterizationMask,
    pipeline,
    fragmentMaskUniformBuffer
  );

  t.CheckColorAttachmentResult(
    color,
    sampleCount,
    rasterizationMask,
    sampleMask,
    fragmentShaderOutputMask
  );

  t.CheckDepthStencilResult(
    'depth-only',
    depthStencil,
    sampleCount,
    rasterizationMask,
    sampleMask,
    fragmentShaderOutputMask
  );

  t.CheckDepthStencilResult(
    'stencil-only',
    depthStencil,
    sampleCount,
    rasterizationMask,
    sampleMask,
    fragmentShaderOutputMask
  );
});

g.test('alpha_to_coverage_mask').
desc(
  `
Test that alpha_to_coverage_mask is working properly with the alpha output of color target[0].

- for sampleCount = 4, alphaToCoverageEnabled = true and various combinations of:
  - rasterization masks
  - increasing alpha0 values of the color0 output including { < 0, = 0, = 1/16, = 2/16, ..., = 15/16, = 1, > 1 }
  - alpha1 values of the color1 output = { 0, 0.5, 1.0 }.
- test that for a single pixel in { color0, color1 } { color0, depth, stencil } output the final sample mask is applied to it, moreover:
  - if alpha0 is 0.0 or less then alpha to coverage mask is 0x0,
  - if alpha0 is 1.0 or greater then alpha to coverage mask is 0xFFFFFFFF,
  - that the number of bits in the alpha to coverage mask is non-decreasing,
  - that the computation of alpha to coverage mask doesn't depend on any other color output than color0,
  - (not included in the spec): that once a sample is included in the alpha to coverage sample mask
    it will be included for any alpha greater than or equal to the current value.

The algorithm of producing the alpha-to-coverage mask is platform-dependent. The test draws a different color
at each sample point. for any two alpha values (alpha and alpha') where 0 < alpha' < alpha < 1, the color values (color and color') must satisfy
color' <= color.
`
).
params((u) =>
u.
combine('interpolated', [false, true]).
combine('sampleCount', [4]).
expand('rasterizationMask', function* (p) {
  const maxMask = 2 ** p.sampleCount - 1;
  for (let i = 0; i <= maxMask; i++) {
    yield i;
  }
}).
beginSubcases().
combine('alpha1', [0.0, 0.5, 1.0])
).
fn(async (t) => {
  const { sampleCount, rasterizationMask, alpha1 } = t.params;
  const sampleMask = 0xffffffff;

  const alphaValues = new Float32Array(4); // [alpha0, alpha1, 0, 0]
  const alphaValueUniformBuffer = t.device.createBuffer({
    size: alphaValues.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
  });
  t.trackForCleanup(alphaValueUniformBuffer);

  const module = t.device.createShaderModule({ code: kSampleMaskTestShader });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module, entryPoint: 'vmain' },
    fragment: {
      module,
      entryPoint: `fmain__alpha_to_coverage_mask__${t.params.interpolated ? 'interp' : 'flat'}`,
      targets: [{ format }, { format }]
    },
    primitive: { topology: 'triangle-list' },
    multisample: {
      count: sampleCount,
      mask: sampleMask,
      alphaToCoverageEnabled: true
    },
    depthStencil: {
      format: depthStencilFormat,
      depthWriteEnabled: true,
      depthCompare: 'always',

      stencilFront: {
        compare: 'always',
        passOp: 'replace'
      },
      stencilBack: {
        compare: 'always',
        passOp: 'replace'
      }
    }
  });

  // { < 0, = 0, = 1/16, = 2/16, ..., = 15/16, = 1, > 1 }
  const alpha0ParamsArray = [-0.1, ...range(16, (i) => i / 16), 1.0, 1.1];

  const colorResultPromises = [];
  const depthResultPromises = [];
  const stencilResultPromises = [];

  for (const alpha0 of alpha0ParamsArray) {
    alphaValues[0] = alpha0;
    alphaValues[1] = alpha1;
    t.device.queue.writeBuffer(alphaValueUniformBuffer, 0, alphaValues);

    const { color, depthStencil } = t.GetTargetTexture(
      sampleCount,
      rasterizationMask,
      pipeline,
      alphaValueUniformBuffer,
      2
    );

    const colorBuffer = t.copySinglePixelTextureToBufferUsingComputePass(
      TypeF32, // correspond to 'rgba8unorm' format
      4,
      color.createView(),
      sampleCount
    );
    const colorResult = t.readGPUBufferRangeTyped(colorBuffer, {
      type: Float32Array,
      typedLength: colorBuffer.size / Float32Array.BYTES_PER_ELEMENT
    });
    colorResultPromises.push(colorResult);

    const depthBuffer = t.copySinglePixelTextureToBufferUsingComputePass(
      TypeF32, // correspond to 'depth24plus-stencil8' format
      1,
      depthStencil.createView({ aspect: 'depth-only' }),
      sampleCount
    );
    const depthResult = t.readGPUBufferRangeTyped(depthBuffer, {
      type: Float32Array,
      typedLength: depthBuffer.size / Float32Array.BYTES_PER_ELEMENT
    });
    depthResultPromises.push(depthResult);

    const stencilBuffer = t.copySinglePixelTextureToBufferUsingComputePass(
      TypeU32, // correspond to 'depth24plus-stencil8' format
      1,
      depthStencil.createView({ aspect: 'stencil-only' }),
      sampleCount
    );
    const stencilResult = t.readGPUBufferRangeTyped(stencilBuffer, {
      type: Uint32Array,
      typedLength: stencilBuffer.size / Uint32Array.BYTES_PER_ELEMENT
    });
    stencilResultPromises.push(stencilResult);
  }

  const resultsArray = await Promise.all([
  Promise.all(colorResultPromises),
  Promise.all(depthResultPromises),
  Promise.all(stencilResultPromises)]
  );

  const checkResults = (
  results,
  getExpectedDataFn,








  positiveCorrelation) =>
  {
    for (let i = 0; i < results.length; i++) {
      const result = results[i];
      const alpha0 = alpha0ParamsArray[i];

      if (alpha0 <= 0) {
        const expected = getExpectedDataFn(sampleCount, rasterizationMask, sampleMask, 0x0);
        const check = checkElementsEqual(result.data, expected);
        t.expectOK(check);
      } else if (alpha0 >= 1) {
        const expected = getExpectedDataFn(
          sampleCount,
          rasterizationMask,
          sampleMask,
          0xffffffff
        );
        const check = checkElementsEqual(result.data, expected);
        t.expectOK(check);
      } else {
        assert(i > 0);
        const prevResult = results[i - 1];
        const check = checkElementsPassPredicate(
          result.data,
          (index, value) =>
          positiveCorrelation ?
          value >= prevResult.data[index] :
          value <= prevResult.data[index],
          {}
        );
        t.expectOK(check);
      }
    }

    for (const result of results) {
      result.cleanup();
    }
  };

  // Check color results
  checkResults(resultsArray[0], getExpectedColorData, true);

  // Check depth results
  checkResults(resultsArray[1], getExpectedDepthData, false);

  // Check stencil results
  checkResults(resultsArray[2], getExpectedStencilData, true);
});