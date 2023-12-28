/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests the behavior of different filtering modes in minFilter/magFilter/mipmapFilter.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { kAddressModes, kMipmapFilterModes } from '../../../capability_info.js';
import {

  kRenderableColorTextureFormats,
  kTextureFormatInfo } from
'../../../format_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';
import { getTextureCopyLayout } from '../../../util/texture/layout.js';
import { TexelView } from '../../../util/texture/texel_view.js';

// Simple checkerboard 2x2 texture used as a base for the sampling.
const kCheckerTextureSize = 2;
const kCheckerTextureData = [
{ R: 1.0, G: 1.0, B: 1.0, A: 1.0 },
{ R: 0.0, G: 0.0, B: 0.0, A: 1.0 },
{ R: 0.0, G: 0.0, B: 0.0, A: 1.0 },
{ R: 1.0, G: 1.0, B: 1.0, A: 1.0 }];


class FilterModeTest extends TextureTestMixin(GPUTest) {
  runFilterRenderPipeline(
  sampler,
  module,
  format,
  renderSize,
  vertexCount,
  instanceCount)
  {
    const sampleTexture = this.createTextureFromTexelView(
      TexelView.fromTexelsAsColors(format, (coord) => {
        const id = coord.x + coord.y * kCheckerTextureSize;
        return kCheckerTextureData[id];
      }),
      {
        size: [kCheckerTextureSize, kCheckerTextureSize],
        usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST
      }
    );
    const renderTexture = this.device.createTexture({
      format,
      size: renderSize,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
    });
    const pipeline = this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module,
        entryPoint: 'vs_main'
      },
      fragment: {
        module,
        entryPoint: 'fs_main',
        targets: [{ format }]
      }
    });
    const bindgroup = this.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      { binding: 0, resource: sampler },
      { binding: 1, resource: sampleTexture.createView() }]

    });
    const commandEncoder = this.device.createCommandEncoder();
    const renderPass = commandEncoder.beginRenderPass({
      colorAttachments: [
      {
        view: renderTexture.createView(),
        clearValue: [0, 0, 0, 0],
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    renderPass.setPipeline(pipeline);
    renderPass.setBindGroup(0, bindgroup);
    renderPass.draw(vertexCount, instanceCount);
    renderPass.end();
    this.device.queue.submit([commandEncoder.finish()]);
    return renderTexture;
  }
}

export const g = makeTestGroup(FilterModeTest);



/* For filter mode 'nearest', we need to check a 6x6 of pixels because 4x4s are identical when using
 * address mode 'clamp-to-edge' and 'mirror-repeat'. The minFilter and magFilter tests are setup so
 * that they both render the same results. (See the respective test for details.) The following
 * table shows the expected results:
 *                                                u
 *
 *                               repeat     clamp-to-edge  mirror-repeat
 *
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *                           │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *             repeat        │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *                           │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *  v       clamp-to-edge    │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *
 *                           │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
 *          mirror-repeat    │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │ │█│ │█│ │█│  │ │ │ │█│█│█│  │█│ │ │█│█│ │
 *                           │█│ │█│ │█│ │  │█│█│█│ │ │ │  │ │█│█│ │ │█│
*/
const kNearestRenderSize = 6;
const kNearestRenderDim = [kNearestRenderSize, kNearestRenderSize];
const kNearestURepeatVRepeat = [
[1, 0, 1, 0, 1, 0],
[0, 1, 0, 1, 0, 1],
[1, 0, 1, 0, 1, 0],
[0, 1, 0, 1, 0, 1],
[1, 0, 1, 0, 1, 0],
[0, 1, 0, 1, 0, 1]];

const kNearestURepeatVClamped = [
[1, 0, 1, 0, 1, 0],
[1, 0, 1, 0, 1, 0],
[1, 0, 1, 0, 1, 0],
[0, 1, 0, 1, 0, 1],
[0, 1, 0, 1, 0, 1],
[0, 1, 0, 1, 0, 1]];

const kNearestURepeatVMirror = [
[0, 1, 0, 1, 0, 1],
[1, 0, 1, 0, 1, 0],
[1, 0, 1, 0, 1, 0],
[0, 1, 0, 1, 0, 1],
[0, 1, 0, 1, 0, 1],
[1, 0, 1, 0, 1, 0]];

const kNearestUClampedVRepeat = [
[1, 1, 1, 0, 0, 0],
[0, 0, 0, 1, 1, 1],
[1, 1, 1, 0, 0, 0],
[0, 0, 0, 1, 1, 1],
[1, 1, 1, 0, 0, 0],
[0, 0, 0, 1, 1, 1]];

const kNearestUClampedVClamped = [
[1, 1, 1, 0, 0, 0],
[1, 1, 1, 0, 0, 0],
[1, 1, 1, 0, 0, 0],
[0, 0, 0, 1, 1, 1],
[0, 0, 0, 1, 1, 1],
[0, 0, 0, 1, 1, 1]];

const kNearestUClampedVMirror = [
[0, 0, 0, 1, 1, 1],
[1, 1, 1, 0, 0, 0],
[1, 1, 1, 0, 0, 0],
[0, 0, 0, 1, 1, 1],
[0, 0, 0, 1, 1, 1],
[1, 1, 1, 0, 0, 0]];

const kNearestUMirrorVRepeat = [
[0, 1, 1, 0, 0, 1],
[1, 0, 0, 1, 1, 0],
[0, 1, 1, 0, 0, 1],
[1, 0, 0, 1, 1, 0],
[0, 1, 1, 0, 0, 1],
[1, 0, 0, 1, 1, 0]];

const kNearestUMirrorVClamped = [
[0, 1, 1, 0, 0, 1],
[0, 1, 1, 0, 0, 1],
[0, 1, 1, 0, 0, 1],
[1, 0, 0, 1, 1, 0],
[1, 0, 0, 1, 1, 0],
[1, 0, 0, 1, 1, 0]];

const kNearestUMirrorVMirror = [
[1, 0, 0, 1, 1, 0],
[0, 1, 1, 0, 0, 1],
[0, 1, 1, 0, 0, 1],
[1, 0, 0, 1, 1, 0],
[1, 0, 0, 1, 1, 0],
[0, 1, 1, 0, 0, 1]];


/* For filter mode 'linear', the tests samples 16 points (to create a 4x4) on what the effective 8x8
 * expanded texture via the address modes looks like (see table below for what those look like). The
 * sample points are selected such that no combination of address modes result in the same render.
 * There is exactly one sample point in each sub 2x2 of the 8x8 texture, thereby yielding the 4x4
 * result. Note that sampling from the 8x8 texture instead of the 6x6 texture is necessary because
 * that allows us to keep the results in powers of 2 to minimize floating point errors on different
 * backends.
 *
 * The 8x8 effective textures:
 *                                                  u
 *
 *                                repeat          clamp-to-edge      mirror-repeat
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *             repeat        │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *  v       clamp-to-edge    │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *          mirror-repeat    │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │█│ │█│ │█│ │█│ │  │ │ │ │ │█│█│█│█│  │█│█│ │ │█│█│ │ │
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *                           │ │█│ │█│ │█│ │█│  │█│█│█│█│ │ │ │ │  │ │ │█│█│ │ │█│█│
 *
 *
 * Sample points:
 *   The sample points are always at a 25% corner of a pixel such that the contributions come from
 *   the 2x2 (doubly outlined) with ratios 1/16, 3/16, or 9/16.
 *                                    ╔══╤══╦══╤══╦══╤══╦══╤══╗
 *                                    ║  │  ║  │  ║  │  ║  │  ║
 *                                    ╟──┼──╫──┼──╫──┼──╫──┼──╢
 *                                    ║  │▘ ║ ▝│  ║  │▘ ║ ▝│  ║
 *                                    ╠══╪══╬══╪══╬══╪══╬══╪══╣
 *                                    ║  │  ║  │  ║  │  ║  │  ║
 *                                    ╟──┼──╫──┼──╫──┼──╫──┼──╢
 *                                    ║  │▘ ║ ▝│  ║  │▘ ║ ▝│  ║
 *                                    ╠══╪══╬══╪══╬══╪══╬══╪══╣
 *                                    ║  │▖ ║ ▗│  ║  │▖ ║ ▗│  ║
 *                                    ╟──┼──╫──┼──╫──┼──╫──┼──╢
 *                                    ║  │  ║  │  ║  │  ║  │  ║
 *                                    ╠══╪══╬══╪══╬══╪══╬══╪══╣
 *                                    ║  │▖ ║ ▗│  ║  │▖ ║ ▗│  ║
 *                                    ╟──┼──╫──┼──╫──┼──╫──┼──╢
 *                                    ║  │  ║  │  ║  │  ║  │  ║
 *                                    ╚══╧══╩══╧══╩══╧══╩══╧══╝
 */
const kLinearRenderSize = 4;
const kLinearRenderDim = [kLinearRenderSize, kLinearRenderSize];
const kLinearURepeatVRepeat = [
[10, 6, 10, 6],
[10, 6, 10, 6],
[6, 10, 6, 10],
[6, 10, 6, 10]];

const kLinearURepeatVClamped = [
[12, 4, 12, 4],
[12, 4, 12, 4],
[4, 12, 4, 12],
[4, 12, 4, 12]];

const kLinearURepeatVMirror = [
[4, 12, 4, 12],
[12, 4, 12, 4],
[4, 12, 4, 12],
[12, 4, 12, 4]];

const kLinearUClampedVRepeat = [
[12, 12, 4, 4],
[12, 12, 4, 4],
[4, 4, 12, 12],
[4, 4, 12, 12]];

const kLinearUClampedVClamped = [
[16, 16, 0, 0],
[16, 16, 0, 0],
[0, 0, 16, 16],
[0, 0, 16, 16]];

const kLinearUClampedVMirror = [
[0, 0, 16, 16],
[16, 16, 0, 0],
[0, 0, 16, 16],
[16, 16, 0, 0]];

const kLinearUMirrorVRepeat = [
[4, 12, 4, 12],
[4, 12, 4, 12],
[12, 4, 12, 4],
[12, 4, 12, 4]];

const kLinearUMirrorVClamped = [
[0, 16, 0, 16],
[0, 16, 0, 16],
[16, 0, 16, 0],
[16, 0, 16, 0]];

const kLinearUMirrorVMirror = [
[16, 0, 16, 0],
[0, 16, 0, 16],
[16, 0, 16, 0],
[0, 16, 0, 16]];




function expectedNearestColors(
format,
addressModeU,
addressModeV)
{
  let expectedColors;
  switch (addressModeU) {
    case 'clamp-to-edge':{
        switch (addressModeV) {
          case 'clamp-to-edge':
            expectedColors = kNearestUClampedVClamped;
            break;
          case 'repeat':
            expectedColors = kNearestUClampedVRepeat;
            break;
          case 'mirror-repeat':
            expectedColors = kNearestUClampedVMirror;
            break;
        }
        break;
      }
    case 'repeat':
      switch (addressModeV) {
        case 'clamp-to-edge':
          expectedColors = kNearestURepeatVClamped;
          break;
        case 'repeat':
          expectedColors = kNearestURepeatVRepeat;
          break;
        case 'mirror-repeat':
          expectedColors = kNearestURepeatVMirror;
          break;
      }
      break;
    case 'mirror-repeat':
      switch (addressModeV) {
        case 'clamp-to-edge':
          expectedColors = kNearestUMirrorVClamped;
          break;
        case 'repeat':
          expectedColors = kNearestUMirrorVRepeat;
          break;
        case 'mirror-repeat':
          expectedColors = kNearestUMirrorVMirror;
          break;
      }
      break;
  }
  return TexelView.fromTexelsAsColors(format, (coord) => {
    const c = expectedColors[coord.y][coord.x];
    return { R: c, G: c, B: c, A: 1.0 };
  });
}
function expectedLinearColors(
format,
addressModeU,
addressModeV)
{
  let expectedColors;
  switch (addressModeU) {
    case 'clamp-to-edge':{
        switch (addressModeV) {
          case 'clamp-to-edge':
            expectedColors = kLinearUClampedVClamped;
            break;
          case 'repeat':
            expectedColors = kLinearUClampedVRepeat;
            break;
          case 'mirror-repeat':
            expectedColors = kLinearUClampedVMirror;
            break;
        }
        break;
      }
    case 'repeat':
      switch (addressModeV) {
        case 'clamp-to-edge':
          expectedColors = kLinearURepeatVClamped;
          break;
        case 'repeat':
          expectedColors = kLinearURepeatVRepeat;
          break;
        case 'mirror-repeat':
          expectedColors = kLinearURepeatVMirror;
          break;
      }
      break;
    case 'mirror-repeat':
      switch (addressModeV) {
        case 'clamp-to-edge':
          expectedColors = kLinearUMirrorVClamped;
          break;
        case 'repeat':
          expectedColors = kLinearUMirrorVRepeat;
          break;
        case 'mirror-repeat':
          expectedColors = kLinearUMirrorVMirror;
          break;
      }
      break;
  }
  return TexelView.fromTexelsAsColors(format, (coord) => {
    const c = expectedColors[coord.y][coord.x];
    return { R: c / 16, G: c / 16, B: c / 16, A: 1.0 };
  });
}
function expectedColors(
format,
filterMode,
addressModeU,
addressModeV)
{
  switch (filterMode) {
    case 'nearest':
      return expectedNearestColors(format, addressModeU, addressModeV);
    case 'linear':
      return expectedLinearColors(format, addressModeU, addressModeV);
  }
}

/* For the magFilter tests, each rendered pixel is an instanced quad such that the center of the
 * quad coincides with the center of the pixel. The uv coordinates for each quad are shifted
 * according to the test so that the center of the quad is at the point we want to sample.
 *
 * For the grid offset logic, see this codelab for reference:
 *   https://codelabs.developers.google.com/your-first-webgpu-app#4
 */

/* The following diagram shows the UV shift (almost to scale) for what the pixel at cell (0,0) looks
 * like w.r.t the UV of the texture if we just mapped the entire 2x2 texture to the quad. Note that
 * the square representing the mapped location on the bottom left is actually slighly smaller than a
 * pixel in order to ensure that we are magnifying the texture and hence using the magFilter. It
 * should be fairly straightforwards to derive that for each pixel, we are shifting (.5, -.5) from
 * the picture.
 *
 *                    ┌─┬─┬─┬─┬─┬─┐
 *                    ├─┼─┼─┼─┼─┼─┤ (0,0) (1,0)
 *                    ├─┼─╔═╪═╗─┼─┤    ╔═══╗
 *                    ├─┼─╫─┼─╫─┼─┤    ║─┼─║
 *                    ├─┼─╚═╪═╝─┼─┤    ╚═══╝       (-.875,1.625) (-.625,1.625)
 *                    ╔═╗─┼─┼─┼─┼─┤ (0,1) (1,1)                ╔═╗
 *                    ╚═╝─┴─┴─┴─┴─┘                            ╚═╝
 *                                                 (-.875,1.875) (-.625,1.875)
 */
g.test('magFilter,nearest').
desc(
  `
  Test that for filterable formats, magFilter 'nearest' mode correctly modifies the sampling.
    - format= {<filterable formats>}
    - addressModeU= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
    - addressModeV= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
filter((t) => {
  return (
    kTextureFormatInfo[t.format].color.type === 'float' ||
    kTextureFormatInfo[t.format].color.type === 'unfilterable-float');

}).
beginSubcases().
combine('addressModeU', kAddressModes).
combine('addressModeV', kAddressModes)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  if (kTextureFormatInfo[t.params.format].color.type === 'unfilterable-float') {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  }
}).
fn((t) => {
  const { format, addressModeU, addressModeV } = t.params;
  const sampler = t.device.createSampler({
    addressModeU,
    addressModeV,
    magFilter: 'nearest'
  });
  const module = t.device.createShaderModule({
    code: `
      @group(0) @binding(0) var s : sampler;
      @group(0) @binding(1) var t : texture_2d<f32>;

      struct VertexOut {
        @builtin(position) pos: vec4f,
        @location(0) uv: vec2f,
      };

      @vertex
      fn vs_main(@builtin(vertex_index) vi : u32,
                 @builtin(instance_index) ii: u32) -> VertexOut {
        const grid = vec2f(${kNearestRenderSize}, ${kNearestRenderSize});
        const posBases = array(
          vec2f(1, 1), vec2f(1, -1), vec2f(-1, -1),
          vec2f(1, 1), vec2f(-1, -1), vec2f(-1, 1),
        );
        const uvBases = array(
          vec2f(1., 0.), vec2f(1., 1.), vec2f(0., 1.),
          vec2f(1., 0.), vec2f(0., 1.), vec2f(0., 0.),
        );

        // Compute the offset of instance plane.
        let cell = vec2f(f32(ii) % grid.x, floor(f32(ii) / grid.y));
        let cellOffset = cell / grid * 2;
        let pos = (posBases[vi] + 1) / grid - 1 + cellOffset;

        // Compute the offset of the UVs.
        let uvBase = uvBases[vi] * 0.25 + vec2f(-0.875, 1.625);
        const uvPerPixelOffset = vec2f(0.5, -0.5);
        return VertexOut(vec4f(pos, 0.0, 1.0), uvBase + uvPerPixelOffset * cell);
      }

      @fragment
      fn fs_main(@location(0) uv : vec2f) -> @location(0) vec4f {
        return textureSample(t, s, uv);
      }
      `
  });
  const vertexCount = 6;
  const instanceCount = kNearestRenderDim.reduce((sink, current) => sink * current);
  const render = t.runFilterRenderPipeline(
    sampler,
    module,
    format,
    kNearestRenderDim,
    vertexCount,
    instanceCount
  );
  t.expectTexelViewComparisonIsOkInTexture(
    { texture: render },
    expectedColors(format, 'nearest', addressModeU, addressModeV),
    kNearestRenderDim
  );
});

/* The following diagram shows the UV shift (almost to scale) for what the pixel at cell (0,0) (the
 * dark square) looks like w.r.t the UV of the texture if we just mapped the entire 2x2 texture to
 * the quad. The other small squares represent the other locations that we are sampling the texture
 * at. The offsets are defined in the shader.
 *
 *             ┌────┬────┬────┬────┬────┬────┬────┬────┐
 *             │    │    │    │    │    │    │    │    │
 *             │    │    │    │    │    │    │    │    │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤
 *             │    │□   │   □│    │    │□   │   □│    │
 *             │    │    │    │    │    │    │    │    │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤
 *             │    │    │    │    │    │    │    │    │
 *             │    │    │    │    │    │    │    │    │       (0,0)     (1,0)
 *             ├────┼────┼────╔════╪════╗────┼────┼────┤         ╔═════════╗
 *             │    │□   │   □║    │    ║□   │   □│    │         ║    │    ║
 *             │    │    │    ║    │    ║    │    │    │         ║    │    ║
 *             ├────┼────┼────╫────┼────╫────┼────┼────┤         ║────┼────║
 *             │    │    │    ║    │    ║    │    │    │         ║    │    ║
 *             │    │□   │   □║    │    ║□   │   □│    │         ║    │    ║
 *             ├────┼────┼────╚════╪════╝────┼────┼────┤         ╚═════════╝
 *             │    │    │    │    │    │    │    │    │       (0,1)     (1,1)
 *             │    │    │    │    │    │    │    │    │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤
 *             │    │    │    │    │    │    │    │    │    (-1,1.75) (-.75,1.75)
 *             │    │■   │   □│    │    │□   │   □│    │             ■
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤       (-1,2) (-.75,2)
 *             │    │    │    │    │    │    │    │    │
 *             │    │    │    │    │    │    │    │    │
 *             └────┴────┴────┴────┴────┴────┴────┴────┘
 */
g.test('magFilter,linear').
desc(
  `
  Test that for filterable formats, magFilter 'linear' mode correctly modifies the sampling.
    - format= {<filterable formats>}
    - addressModeU= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
    - addressModeV= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
filter((t) => {
  return (
    kTextureFormatInfo[t.format].color.type === 'float' ||
    kTextureFormatInfo[t.format].color.type === 'unfilterable-float');

}).
beginSubcases().
combine('addressModeU', kAddressModes).
combine('addressModeV', kAddressModes)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  if (kTextureFormatInfo[t.params.format].color.type === 'unfilterable-float') {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  }
}).
fn((t) => {
  const { format, addressModeU, addressModeV } = t.params;
  const sampler = t.device.createSampler({
    addressModeU,
    addressModeV,
    magFilter: 'linear'
  });
  const module = t.device.createShaderModule({
    code: `
      @group(0) @binding(0) var s : sampler;
      @group(0) @binding(1) var t : texture_2d<f32>;

      struct VertexOut {
        @builtin(position) pos: vec4f,
        @location(0) uv: vec2f,
      };

      @vertex
      fn vs_main(@builtin(vertex_index) vi : u32,
                 @builtin(instance_index) ii: u32) -> VertexOut {
        const grid = vec2f(${kLinearRenderSize}, ${kLinearRenderSize});
        const posBases = array(
          vec2f(1, 1), vec2f(1, -1), vec2f(-1, -1),
          vec2f(1, 1), vec2f(-1, -1), vec2f(-1, 1),
        );
        const uvBases = array(
          vec2f(1., 0.), vec2f(1., 1.), vec2f(0., 1.),
          vec2f(1., 0.), vec2f(0., 1.), vec2f(0., 0.),
        );

        // Compute the offset of instance plane.
        let cell = vec2f(f32(ii) % grid.x, floor(f32(ii) / grid.y));
        let cellOffset = cell / grid * 2;
        let pos = (posBases[vi] + 1) / grid - 1 + cellOffset;

        // Compute the offset of the UVs.
        const uOffsets = array(0., 0.75, 2., 2.75);
        const vOffsets = array(0., 1., 1.75, 2.75);
        let uvBase = uvBases[vi] * 0.25 + vec2f(-1., 1.75);
        let uvPixelOffset = vec2f(uOffsets[u32(cell.x)], -vOffsets[u32(cell.y)]);
        return VertexOut(vec4f(pos, 0.0, 1.0), uvBase + uvPixelOffset);
      }

      @fragment
      fn fs_main(@location(0) uv : vec2f) -> @location(0) vec4f {
        return textureSample(t, s, uv);
      }
      `
  });
  const vertexCount = 6;
  const instanceCount = kLinearRenderDim.reduce((sink, current) => sink * current);
  const render = t.runFilterRenderPipeline(
    sampler,
    module,
    format,
    kLinearRenderDim,
    vertexCount,
    instanceCount
  );
  t.expectTexelViewComparisonIsOkInTexture(
    { texture: render },
    expectedColors(format, 'linear', addressModeU, addressModeV),
    kLinearRenderDim
  );
});

/* For the minFilter tests, each rendered pixel is a small instanced quad that is UV mapped such
 * that it is either the 6x6 or 8x8 textures from above. Each quad in each cell is then offsetted
 * and scaled so that the target sample point coincides with the center of the pixel and the texture
 * is significantly smaller than the pixel to force minFilter mode.
 *
 * For the grid offset logic, see this codelab for reference:
 *   https://codelabs.developers.google.com/your-first-webgpu-app#4
 */

/* The following diagram depicts a single pixel and the sub-pixel sized 6x6 textured quad. The
 * distances shown in the diagram are pre-grid transformation and relative to the quad. Notice that
 * for cell (0,0) marked with an x, we need to offset the center by (5/12,5/12), and per cell, the
 * offset is (-1/6, -1/6).
 *
 *
 *              ┌───────────────────────────────────────────────┐
 *              │                                               │
 *              │                                               │
 *              │                                               │
 *              │                                               │
 *              │                                               │
 *              │           ┌───┬───┬───┬───┬───┬───┐           │
 *              │           │   │   │   │   │   │   │           │
 *              │           ├───┼───┼───┼───┼───┼───┤           │
 *              │           │   │   │   │   │   │   │           │
 *              │           ├───┼───┼───┼───┼───┼───┤           │
 *              │           │   │   │   │   │   │   │           │
 *              │           ├───┼───┼───x───┼───┼───┤           │         ┐
 *              │           │   │   │   │   │   │   │           │         │
 *              │           ├───┼───┼───┼───┼───┼───┤           │         │ 5/12
 *              │           │   │   │   │   │   │   │           │ ┐       │
 *              │           ├───┼───┼───┼───┼───┼───┤           │ │ 1/6   │
 *              │           │ x │   │   │   │   │   │           │ ┘       ┘
 *              │           └───┴───┴───┴───┴───┴───┘           │
 *              │                                               │
 *              │                                               │
 *              │                                               │
 *              │                                               │
 *              │                                               │
 *              └───────────────────────────────────────────────┘
 */
g.test('minFilter,nearest').
desc(
  `
  Test that for filterable formats, minFilter 'nearest' mode correctly modifies the sampling.
    - format= {<filterable formats>}
    - addressModeU= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
    - addressModeV= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
filter((t) => {
  return (
    kTextureFormatInfo[t.format].color.type === 'float' ||
    kTextureFormatInfo[t.format].color.type === 'unfilterable-float');

}).
beginSubcases().
combine('addressModeU', kAddressModes).
combine('addressModeV', kAddressModes)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  if (kTextureFormatInfo[t.params.format].color.type === 'unfilterable-float') {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  }
}).
fn((t) => {
  const { format, addressModeU, addressModeV } = t.params;
  const sampler = t.device.createSampler({
    addressModeU,
    addressModeV,
    minFilter: 'nearest'
  });
  const module = t.device.createShaderModule({
    code: `
      @group(0) @binding(0) var s : sampler;
      @group(0) @binding(1) var t : texture_2d<f32>;

      struct VertexOut {
        @builtin(position) pos: vec4f,
        @location(0) uv: vec2f,
      };

      @vertex
      fn vs_main(@builtin(vertex_index) vi : u32,
                 @builtin(instance_index) ii: u32) -> VertexOut {
        const grid = vec2f(${kNearestRenderSize}, ${kNearestRenderSize});
        const posBases = array(
          vec2f(.5, .5), vec2f(.5, -.5), vec2f(-.5, -.5),
          vec2f(.5, .5), vec2f(-.5, -.5), vec2f(-.5, .5),
        );
        // Choose UVs so that the quad ends up being the 6x6 texture.
        const uvBases = array(
          vec2f(2., -1.), vec2f(2., 2.), vec2f(-1., 2.),
          vec2f(2., -1.), vec2f(-1., 2.), vec2f(-1., -1.),
        );

        let cell = vec2f(f32(ii) % grid.x, floor(f32(ii) / grid.y));

        // Compute the offset of instance plane (pre-grid transformation).
        const constantPlaneOffset = vec2f(5. / 12., 5. / 12.);
        const perPixelOffset = vec2f(1. / 6., 1. / 6.);
        let posBase = posBases[vi] + constantPlaneOffset - perPixelOffset * cell;

        // Apply the grid transformation.
        let cellOffset = cell / grid * 2;
        let absPos = (posBase + 1) / grid - 1 + cellOffset;

        return VertexOut(vec4f(absPos, 0.0, 1.0), uvBases[vi]);
      }

      @fragment
      fn fs_main(@location(0) uv : vec2f) -> @location(0) vec4f {
        return textureSample(t, s, uv);
      }
      `
  });
  const vertexCount = 6;
  const instanceCount = kNearestRenderDim.reduce((sink, current) => sink * current);
  const render = t.runFilterRenderPipeline(
    sampler,
    module,
    format,
    kNearestRenderDim,
    vertexCount,
    instanceCount
  );
  t.expectTexelViewComparisonIsOkInTexture(
    { texture: render },
    expectedColors(format, 'nearest', addressModeU, addressModeV),
    kNearestRenderDim
  );
});

/* The following diagram shows the sub-pixel quad and the relative distances between the sample
 * points and the origin. The pixel is not shown in this diagram but is a 2x bounding box around the
 * quad similar to the one in the diagram for minFilter,nearest above. The dark square is where the
 * cell (0,0) is, and the offsets are all relative to that point.
 *
 *                        11/32
 *                   ┌─────────────┐
 *
 *                     3/16      5/16       3/16
 *                   ┌───────┬───────────┬───────┐
 *
 *             ┌────┬────┬────┬────┬────┬────┬────┬────┐
 *             │    │    │    │    │    │    │    │    │
 *             │    │    │    │    │    │    │    │    │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤
 *             │    │□   │   □│    │    │□   │   □│    │  ┐
 *             │    │    │    │    │    │    │    │    │  │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤  │
 *             │    │    │    │    │    │    │    │    │  │  1/4
 *             │    │    │    │    │    │    │    │    │  │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤  │
 *             │    │□   │   □│    │    │□   │   □│    │  ┤
 *             │    │    │    │    │    │    │    │    │  │
 *             ├────┼────┼────┼────x────┼────┼────┼────┤  │  3/16    ┐
 *             │    │    │    │    │    │    │    │    │  │          │
 *             │    │□   │   □│    │    │□   │   □│    │  ┤          │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤  │          │
 *             │    │    │    │    │    │    │    │    │  │          │  11/32
 *             │    │    │    │    │    │    │    │    │  │  1/4     │
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤  │          │
 *             │    │    │    │    │    │    │    │    │  │          │
 *             │    │■   │   □│    │    │□   │   □│    │  ┘          ┘
 *             ├────┼────┼────┼────┼────┼────┼────┼────┤
 *             │    │    │    │    │    │    │    │    │
 *             │    │    │    │    │    │    │    │    │
 *             └────┴────┴────┴────┴────┴────┴────┴────┘
 */
g.test('minFilter,linear').
desc(
  `
  Test that for filterable formats, minFilter 'linear' mode correctly modifies the sampling.
    - format= {<filterable formats>}
    - addressModeU= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
    - addressModeV= {'clamp-to-edge', 'repeat', 'mirror-repeat'}
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
filter((t) => {
  return (
    kTextureFormatInfo[t.format].color.type === 'float' ||
    kTextureFormatInfo[t.format].color.type === 'unfilterable-float');

}).
beginSubcases().
combine('addressModeU', kAddressModes).
combine('addressModeV', kAddressModes)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  if (kTextureFormatInfo[t.params.format].color.type === 'unfilterable-float') {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  }
}).
fn((t) => {
  const { format, addressModeU, addressModeV } = t.params;
  const sampler = t.device.createSampler({
    addressModeU,
    addressModeV,
    minFilter: 'linear'
  });
  const module = t.device.createShaderModule({
    code: `
      @group(0) @binding(0) var s : sampler;
      @group(0) @binding(1) var t : texture_2d<f32>;

      struct VertexOut {
        @builtin(position) pos: vec4f,
        @location(0) uv: vec2f,
      };

      @vertex
      fn vs_main(@builtin(vertex_index) vi : u32,
                 @builtin(instance_index) ii: u32) -> VertexOut {
        const grid = vec2f(${kLinearRenderSize}, ${kLinearRenderSize});
        const posBases = array(
          vec2f(.5, .5), vec2f(.5, -.5), vec2f(-.5, -.5),
          vec2f(.5, .5), vec2f(-.5, -.5), vec2f(-.5, .5),
        );
        // Choose UVs so that the quad ends up being the 8x8 texture.
        const uvBases = array(
          vec2f(2.5, -1.5), vec2f(2.5, 2.5), vec2f(-1.5, 2.5),
          vec2f(2.5, -1.5), vec2f(-1.5, 2.5), vec2f(-1.5, -1.5),
        );

        let cell = vec2f(f32(ii) % grid.x, floor(f32(ii) / grid.y));

        // Compute the offset of instance plane (pre-grid transformation).
        const constantPlaneOffset = vec2f(11. / 32., 11. / 32.);
        const xOffsets = array(0., 3. / 16., 1. / 2., 11. / 16.);
        const yOffsets = array(0., 1. / 4., 7. / 16., 11. / 16.);
        let pixelOffset = vec2f(xOffsets[u32(cell.x)], yOffsets[u32(cell.y)]);
        let posBase = posBases[vi] + constantPlaneOffset - pixelOffset;

        // Compute the offset of instance plane.
        let cellOffset = cell / grid * 2;
        let absPos = (posBase + 1) / grid - 1 + cellOffset;

        return VertexOut(vec4f(absPos, 0.0, 1.0), uvBases[vi]);
      }

      @fragment
      fn fs_main(@location(0) uv : vec2f) -> @location(0) vec4f {
        return textureSample(t, s, uv);
      }
      `
  });
  const vertexCount = 6;
  const instanceCount = kLinearRenderDim.reduce((sink, current) => sink * current);
  const render = t.runFilterRenderPipeline(
    sampler,
    module,
    format,
    kLinearRenderDim,
    vertexCount,
    instanceCount
  );
  t.expectTexelViewComparisonIsOkInTexture(
    { texture: render },
    expectedColors(format, 'linear', addressModeU, addressModeV),
    kLinearRenderDim
  );
});

g.test('mipmapFilter').
desc(
  `
  Test that for filterable formats, mipmapFilter modes correctly modifies the sampling.
    - format= {<filterable formats>}
    - filterMode= {'nearest', 'linear'}
  `
).
params((u) =>
u.
combine('format', kRenderableColorTextureFormats).
filter((t) => {
  return (
    kTextureFormatInfo[t.format].color.type === 'float' ||
    kTextureFormatInfo[t.format].color.type === 'unfilterable-float');

}).
beginSubcases().
combine('filterMode', kMipmapFilterModes)
).
beforeAllSubcases((t) => {
  t.skipIfTextureFormatNotSupported(t.params.format);
  if (kTextureFormatInfo[t.params.format].color.type === 'unfilterable-float') {
    t.selectDeviceOrSkipTestCase('float32-filterable');
  }
}).
fn((t) => {
  const { format, filterMode } = t.params;
  // Takes a 8x8/4x4 mipmapped texture and renders it on multiple quads with different UVs such
  // that each instanced quad from left to right emulates moving the quad further and further from
  // the camera. Each quad is then rendered to a single pixel in a 1-dimensional texture. Since
  // the 8x8 is fully black and the 4x4 is fully white, we should see the pixels increase in
  // brightness from left to right when sampling linearly, and jump from black to white when
  // sampling for the nearest mip level.
  const kTextureSize = 8;
  const kRenderSize = 8;

  const sampler = t.device.createSampler({
    mipmapFilter: filterMode
  });
  const sampleTexture = t.createTextureFromTexelViewsMultipleMipmaps(
    [
    TexelView.fromTexelsAsColors(format, () => {
      return { R: 0.0, G: 0.0, B: 0.0, A: 1.0 };
    }),
    TexelView.fromTexelsAsColors(format, (_coords) => {
      return { R: 1.0, G: 1.0, B: 1.0, A: 1.0 };
    })],

    {
      size: [kTextureSize, 1],
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST
    }
  );
  const renderTexture = t.device.createTexture({
    format,
    size: [kRenderSize, 1],
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });
  const module = t.device.createShaderModule({
    code: `
      @group(0) @binding(0) var s : sampler;
      @group(0) @binding(1) var t : texture_2d<f32>;

      struct VertexOut {
        @builtin(position) pos: vec4f,
        @location(0) uv: vec2f,
      };

      @vertex
      fn vs_main(@builtin(vertex_index) vi : u32,
                 @builtin(instance_index) ii: u32) -> VertexOut {
        const grid = vec2f(${kRenderSize}., 1.);
        const pos = array(
          vec2f( 1.0,  1.0), vec2f( 1.0, -1.0), vec2f(-1.0, -1.0),
          vec2f( 1.0,  1.0), vec2f(-1.0, -1.0), vec2f(-1.0,  1.0),
        );
        const uv = array(
          vec2f(1., 0.), vec2f(1., 1.), vec2f(0., 1.),
          vec2f(1., 0.), vec2f(0., 1.), vec2f(0., 0.),
        );

        // Compute the offset of the plane.
        let cell = vec2f(f32(ii) % grid.x, 0.);
        let cellOffset = cell / grid * 2;
        let absPos = (pos[vi] + 1) / grid - 1 + cellOffset;
        let uvFactor = (1. / 8.) * (1 + (f32(ii) / (grid.x - 1)));
        return VertexOut(vec4f(absPos, 0.0, 1.0), uv[vi] * uvFactor);
      }

      @fragment
      fn fs_main(@location(0) uv : vec2f) -> @location(0) vec4f {
        return textureSample(t, s, uv);
      }
      `
  });
  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module,
      entryPoint: 'vs_main'
    },
    fragment: {
      module,
      entryPoint: 'fs_main',
      targets: [{ format }]
    }
  });
  const bindgroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: sampler },
    { binding: 1, resource: sampleTexture.createView() }]

  });
  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTexture.createView(),
      clearValue: [0, 0, 0, 0],
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  renderPass.setPipeline(pipeline);
  renderPass.setBindGroup(0, bindgroup);
  renderPass.draw(6, kRenderSize);
  renderPass.end();
  t.device.queue.submit([commandEncoder.finish()]);

  // Since mipmap filtering varies across different backends, we verify that the result exhibits
  // filtered characteristics without strict value equalities via copies to a buffer.
  const buffer = t.copyWholeTextureToNewBufferSimple(renderTexture, 0);
  t.expectGPUBufferValuesPassCheck(
    buffer,
    (actual) => {
      // Convert the buffer to texel view so we can do comparisons.
      const layout = getTextureCopyLayout(format, '2d', [kRenderSize, 1, 1]);
      const view = TexelView.fromTextureDataByReference(format, actual, {
        bytesPerRow: layout.bytesPerRow,
        rowsPerImage: layout.rowsPerImage,
        subrectOrigin: [0, 0, 0],
        subrectSize: [kRenderSize, 1, 1]
      });

      // We only check the R component for the conditions, since all components should be equal if
      // specified in the format.
      switch (filterMode) {
        case 'linear':{
            // For 'linear' mode, we check that the resulting 1d image is monotonically increasing.
            for (let x = 1; x < kRenderSize; x++) {
              const { R: Ri } = view.color({ x: x - 1, y: 0, z: 0 });
              const { R: Rj } = view.color({ x, y: 0, z: 0 });
              if (Ri >= Rj) {
                return Error(
                  'Linear filtering on mipmaps should be a monotonically increasing sequence:\n' +
                  view.toString(
                    { x: 0, y: 0, z: 0 },
                    { width: kRenderSize, height: 1, depthOrArrayLayers: 1 }
                  )
                );
              }
            }
            break;
          }
        case 'nearest':{
            // For 'nearest' mode, we check that the resulting 1d image changes from 0.0 to 1.0
            // exactly once.
            let changes = 0;
            for (let x = 1; x < kRenderSize; x++) {
              const { R: Ri } = view.color({ x: x - 1, y: 0, z: 0 });
              const { R: Rj } = view.color({ x, y: 0, z: 0 });
              if (Ri !== Rj) {
                changes++;
              }
            }
            if (changes !== 1) {
              return Error(
                `Nearest filtering on mipmaps should change exacly once but found (${changes}):\n` +
                view.toString(
                  { x: 0, y: 0, z: 0 },
                  { width: kRenderSize, height: 1, depthOrArrayLayers: 1 }
                )
              );
            }
            break;
          }
      }
      return undefined;
    },
    { srcByteOffset: 0, type: Uint8Array, typedLength: buffer.size }
  );
});