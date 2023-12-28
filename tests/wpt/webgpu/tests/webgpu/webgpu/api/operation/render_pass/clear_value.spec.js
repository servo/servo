/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for render pass clear values.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { assert } from '../../../../common/util/util.js';
import {
  kTextureFormatInfo,
  kDepthStencilFormats,
  depthStencilFormatAspectSize } from
'../../../format_info.js';
import { GPUTest } from '../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('stored').
desc(`Test render pass clear values are stored at the end of an empty pass.`).
unimplemented();

g.test('loaded').
desc(
  `Test render pass clear values are visible during the pass by doing some trivial blending
with the attachment (e.g. add [0,0,0,0] to the color and verify the stored result).`
).
unimplemented();

g.test('srgb').
desc(
  `Test that clear values on '-srgb' type attachments are interpreted as unencoded (linear),
not decoded from srgb to linear.`
).
unimplemented();

g.test('layout').
desc(
  `Test that bind group layouts of the default pipeline layout are correct by passing various
shaders and then checking their computed bind group layouts are compatible with particular bind
groups.`
).
unimplemented();

g.test('stencil_clear_value').
desc(
  `Test that when stencilLoadOp is "clear", the stencil aspect should be correctly cleared by
     GPURenderPassDepthStencilAttachment.stencilClearValue, which will be converted to the type of
     the stencil aspect of view by taking the same number of LSBs as the number of bits in the
     stencil aspect of one texel block of view.`
).
params((u) =>
u.
combine('stencilFormat', kDepthStencilFormats).
combine('stencilClearValue', [0, 1, 0xff, 0x100 + 2, 0x10000 + 3]).
combine('applyStencilClearValueAsStencilReferenceValue', [true, false]).
filter((t) => !!kTextureFormatInfo[t.stencilFormat].stencil)
).
beforeAllSubcases((t) => {
  const { stencilFormat } = t.params;
  const info = kTextureFormatInfo[stencilFormat];
  t.selectDeviceOrSkipTestCase(info.feature);
}).
fn((t) => {
  const { stencilFormat, stencilClearValue, applyStencilClearValueAsStencilReferenceValue } =
  t.params;

  const kSize = [1, 1, 1];
  const colorFormat = 'rgba8unorm';
  const stencilTexture = t.device.createTexture({
    format: stencilFormat,
    size: kSize,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });
  const colorTexture = t.device.createTexture({
    format: colorFormat,
    size: kSize,
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
  });
  const renderPipeline = t.device.createRenderPipeline({
    layout: 'auto',
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
            @fragment
            fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`
      }),
      entryPoint: 'main',
      targets: [{ format: colorFormat }]
    },
    depthStencil: {
      format: stencilFormat,
      depthCompare: 'always',
      depthWriteEnabled: false,
      stencilFront: {
        compare: 'equal'
      },
      stencilBack: {
        compare: 'equal'
      }
    },
    primitive: {
      topology: 'triangle-list'
    }
  });

  const stencilAspectSizeInBytes = depthStencilFormatAspectSize(stencilFormat, 'stencil-only');
  assert(stencilAspectSizeInBytes > 0);
  const expectedStencilValue = stencilClearValue & (stencilAspectSizeInBytes << 8) - 1;

  // StencilReference used in setStencilReference will also be masked to the lowest valid bits, so
  // no matter what we set in the rest high bits that will be masked out (different or same
  // between stencilClearValue and stencilReference), the test will pass if and only if the valid
  // lowest bits are the same.
  const stencilReference = applyStencilClearValueAsStencilReferenceValue ?
  stencilClearValue :
  expectedStencilValue;

  const encoder = t.device.createCommandEncoder();

  const depthStencilAttachment = {
    view: stencilTexture.createView(),
    depthClearValue: 0,
    stencilLoadOp: 'clear',
    stencilStoreOp: 'store',
    stencilClearValue
  };
  if (kTextureFormatInfo[stencilFormat].depth) {
    depthStencilAttachment.depthClearValue = 0;
    depthStencilAttachment.depthLoadOp = 'clear';
    depthStencilAttachment.depthStoreOp = 'store';
  }
  const renderPassEncoder = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorTexture.createView(),
      loadOp: 'clear',
      storeOp: 'store',
      clearValue: [1, 0, 0, 1]
    }],

    depthStencilAttachment
  });
  renderPassEncoder.setPipeline(renderPipeline);
  renderPassEncoder.setStencilReference(stencilReference);
  renderPassEncoder.draw(6);
  renderPassEncoder.end();

  const destinationBuffer = t.device.createBuffer({
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
    size: 4
  });
  t.trackForCleanup(destinationBuffer);
  encoder.copyTextureToBuffer(
    {
      texture: stencilTexture,
      aspect: 'stencil-only'
    },
    {
      buffer: destinationBuffer
    },
    [1, 1, 1]
  );

  t.queue.submit([encoder.finish()]);

  t.expectSingleColor(colorTexture, colorFormat, {
    size: [1, 1, 1],
    exp: { R: 0, G: 1, B: 0, A: 1 }
  });
  t.expectGPUBufferValuesEqual(destinationBuffer, new Uint8Array([expectedStencilValue]));
});