/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests the behavior of anisotropic filtering.

TODO:
Note that anisotropic filtering is never guaranteed to occur, but we might be able to test some
things. If there are no guarantees we can issue warnings instead of failures. Ideas:
  - No *more* than the provided maxAnisotropy samples are used, by testing how many unique
    sample values come out of the sample operation.
  - Check anisotropy is done in the correct direction (by having a 2D gradient and checking we get
    more of the color in the correct direction).
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { checkElementsEqual } from '../../../util/check_contents.js';
import { TexelView } from '../../../util/texture/texel_view.js';


const kRTSize = 16;
const kBytesPerRow = 256;
const xMiddle = kRTSize / 2; // we check the pixel value in the middle of the render target
const kColorAttachmentFormat = 'rgba8unorm';
const kTextureFormat = 'rgba8unorm';
const colors = [
new Uint8Array([0xff, 0x00, 0x00, 0xff]), // miplevel = 0
new Uint8Array([0x00, 0xff, 0x00, 0xff]), // miplevel = 1
new Uint8Array([0x00, 0x00, 0xff, 0xff]) // miplevel = 2
];
const checkerColors = [
new Uint8Array([0xff, 0x00, 0x00, 0xff]),
new Uint8Array([0x00, 0xff, 0x00, 0xff])];


// renders texture a slanted plane placed in a specific way
class SamplerAnisotropicFilteringSlantedPlaneTest extends GPUTest {
  copyRenderTargetToBuffer(rt) {
    const byteLength = kRTSize * kBytesPerRow;
    const buffer = this.createBufferTracked({
      size: byteLength,
      usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
    });

    const commandEncoder = this.device.createCommandEncoder();
    commandEncoder.copyTextureToBuffer(
      { texture: rt, mipLevel: 0, origin: [0, 0, 0] },
      { buffer, bytesPerRow: kBytesPerRow, rowsPerImage: kRTSize },
      { width: kRTSize, height: kRTSize, depthOrArrayLayers: 1 }
    );
    this.queue.submit([commandEncoder.finish()]);

    return buffer;
  }


  async init() {
    await super.init();

    this.pipeline = this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: `
            struct Outputs {
              @builtin(position) Position : vec4<f32>,
              @location(0) fragUV : vec2<f32>,
            };

            @vertex fn main(
              @builtin(vertex_index) VertexIndex : u32) -> Outputs {
              var position : array<vec3<f32>, 6> = array<vec3<f32>, 6>(
                vec3<f32>(-0.5, 0.5, -0.5),
                vec3<f32>(0.5, 0.5, -0.5),
                vec3<f32>(-0.5, 0.5, 0.5),
                vec3<f32>(-0.5, 0.5, 0.5),
                vec3<f32>(0.5, 0.5, -0.5),
                vec3<f32>(0.5, 0.5, 0.5));
              // uv is pre-scaled to mimic repeating tiled texture
              var uv : array<vec2<f32>, 6> = array<vec2<f32>, 6>(
                vec2<f32>(0.0, 0.0),
                vec2<f32>(1.0, 0.0),
                vec2<f32>(0.0, 50.0),
                vec2<f32>(0.0, 50.0),
                vec2<f32>(1.0, 0.0),
                vec2<f32>(1.0, 50.0));
              // draw a slanted plane in a specific way
              let matrix : mat4x4<f32> = mat4x4<f32>(
                vec4<f32>(-1.7320507764816284, 1.8322050568049563e-16, -6.176817699518044e-17, -6.170640314703498e-17),
                vec4<f32>(-2.1211504944260596e-16, -1.496108889579773, 0.5043753981590271, 0.5038710236549377),
                vec4<f32>(0.0, -43.63650894165039, -43.232173919677734, -43.18894577026367),
                vec4<f32>(0.0, 21.693578720092773, 21.789791107177734, 21.86800193786621));

              var output : Outputs;
              output.fragUV = uv[VertexIndex];
              output.Position = matrix * vec4<f32>(position[VertexIndex], 1.0);
              return output;
            }
            `
        }),
        entryPoint: 'main'
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
            @group(0) @binding(0) var sampler0 : sampler;
            @group(0) @binding(1) var texture0 : texture_2d<f32>;

            @fragment fn main(
              @builtin(position) FragCoord : vec4<f32>,
              @location(0) fragUV: vec2<f32>)
              -> @location(0) vec4<f32> {
                return textureSample(texture0, sampler0, fragUV);
            }
            `
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }]
      },
      primitive: { topology: 'triangle-list' }
    });
  }

  // return the render target texture object
  drawSlantedPlane(textureView, sampler) {
    // make sure it's already initialized
    assert(this.pipeline !== undefined);

    const bindGroup = this.device.createBindGroup({
      entries: [
      { binding: 0, resource: sampler },
      { binding: 1, resource: textureView }],

      layout: this.pipeline.getBindGroupLayout(0)
    });

    const colorAttachment = this.createTextureTracked({
      format: kColorAttachmentFormat,
      size: { width: kRTSize, height: kRTSize, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.RENDER_ATTACHMENT
    });
    const colorAttachmentView = colorAttachment.createView();

    const encoder = this.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachmentView,
        clearValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(this.pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.draw(6);
    pass.end();
    this.device.queue.submit([encoder.finish()]);

    return colorAttachment;
  }
}

export const g = makeTestGroup(TextureTestMixin(SamplerAnisotropicFilteringSlantedPlaneTest));

g.test('anisotropic_filter_checkerboard').
desc(
  `Anisotropic filter rendering tests that draws a slanted plane and samples from a texture
    that only has a top level mipmap, the content of which is like a checkerboard.
    We will check the rendering result using sampler with maxAnisotropy values to be
    different from each other, as the sampling rate is different.
    We will also check if those large maxAnisotropy values are clamped so that rendering is the
    same as the supported upper limit say 16.
    A similar webgl demo is at https://jsfiddle.net/yqnbez24`
).
fn(async (t) => {
  // init texture with only a top level mipmap
  const textureSize = 32;
  const texture = t.createTextureTracked({
    mipLevelCount: 1,
    size: { width: textureSize, height: textureSize, depthOrArrayLayers: 1 },
    format: kTextureFormat,
    usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING
  });

  const textureEncoder = t.device.createCommandEncoder();

  const bufferSize = kBytesPerRow * textureSize; // RGBA8 for each pixel (256 > 16 * 4)

  // init checkerboard texture data
  const data = new Uint8Array(bufferSize);
  for (let r = 0; r < textureSize; r++) {
    const o = r * kBytesPerRow;
    for (let c = o, end = o + textureSize * 4; c < end; c += 4) {
      const cid = (r + (c - o) / 4) % 2;
      const color = checkerColors[cid];
      data[c] = color[0];
      data[c + 1] = color[1];
      data[c + 2] = color[2];
      data[c + 3] = color[3];
    }
  }
  const buffer = t.makeBufferWithContents(
    data,
    GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  const bytesPerRow = kBytesPerRow;
  const rowsPerImage = textureSize;

  textureEncoder.copyBufferToTexture(
    {
      buffer,
      bytesPerRow,
      rowsPerImage
    },
    {
      texture,
      mipLevel: 0,
      origin: [0, 0, 0]
    },
    [textureSize, textureSize, 1]
  );

  t.device.queue.submit([textureEncoder.finish()]);

  const textureView = texture.createView();
  const byteLength = kRTSize * kBytesPerRow;
  const results = [];

  for (const maxAnisotropy of [1, 16, 1024]) {
    const sampler = t.device.createSampler({
      magFilter: 'linear',
      minFilter: 'linear',
      mipmapFilter: 'linear',
      maxAnisotropy
    });
    const result = await t.readGPUBufferRangeTyped(
      t.copyRenderTargetToBuffer(t.drawSlantedPlane(textureView, sampler)),
      { type: Uint8Array, typedLength: byteLength }
    );
    results.push(result);
  }

  const check0 = checkElementsEqual(results[0].data, results[1].data);
  if (check0 === undefined) {
    t.warn('Render results with sampler.maxAnisotropy being 1 and 16 should be different.');
  }
  const check1 = checkElementsEqual(results[1].data, results[2].data);
  if (check1 !== undefined) {
    t.expect(
      false,
      'Render results with sampler.maxAnisotropy being 16 and 1024 should be the same.'
    );
  }

  for (const result of results) {
    result.cleanup();
  }
});

g.test('anisotropic_filter_mipmap_color').
desc(
  `Anisotropic filter rendering tests that draws a slanted plane and samples from a texture
    containing mipmaps of different colors. Given the same fragment with dFdx and dFdy for uv being different,
    sampler with bigger maxAnisotropy value tends to bigger mip levels to provide better details.
    We can then look at the color of the fragment to know which mip level is being sampled from and to see
    if it fits expectations.
    A similar webgl demo is at https://jsfiddle.net/t8k7c95o/5/`
).
paramsSimple([
{
  maxAnisotropy: 1,
  _results: [
  { coord: { x: xMiddle, y: 2 }, expected: colors[2] },
  { coord: { x: xMiddle, y: 6 }, expected: [colors[0], colors[1]] }],

  _generateWarningOnly: false
},
{
  maxAnisotropy: 4,
  _results: [
  { coord: { x: xMiddle, y: 2 }, expected: [colors[0], colors[1]] },
  { coord: { x: xMiddle, y: 6 }, expected: colors[0] }],

  _generateWarningOnly: true
}]
).
fn((t) => {
  const texture = t.createTextureFromTexelViewsMultipleMipmaps(
    colors.map((value) => TexelView.fromTexelsAsBytes(kTextureFormat, (_coords) => value)),
    { size: [4, 4, 1], usage: GPUTextureUsage.COPY_DST | GPUTextureUsage.TEXTURE_BINDING }
  );
  const textureView = texture.createView();

  const sampler = t.device.createSampler({
    magFilter: 'linear',
    minFilter: 'linear',
    mipmapFilter: 'linear',
    maxAnisotropy: t.params.maxAnisotropy
  });

  const colorAttachment = t.drawSlantedPlane(textureView, sampler);

  const pixelComparisons = [];
  for (const entry of t.params._results) {
    if (entry.expected instanceof Uint8Array) {
      // equal exactly one color
      pixelComparisons.push({ coord: entry.coord, exp: entry.expected });
    } else {
      // a lerp between two colors
      // MAINTENANCE_TODO: Unify comparison to allow for a strict in-between comparison to support
      //                   this kind of expectation.
      t.expectSinglePixelBetweenTwoValuesIn2DTexture(
        colorAttachment,
        kColorAttachmentFormat,
        entry.coord,
        {
          exp: entry.expected,
          generateWarningOnly: t.params._generateWarningOnly
        }
      );
    }
  }
  t.expectSinglePixelComparisonsAreOkInTexture({ texture: colorAttachment }, pixelComparisons);
});