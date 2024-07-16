/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for the indirect-specific aspects of drawIndirect/drawIndexedIndirect.
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import {
  kDrawIndirectParametersSize,
  kDrawIndexedIndirectParametersSize } from
'../../../capability_info.js';
import { GPUTest, TextureTestMixin } from '../../../gpu_test.js';

const filled = new Uint8Array([0, 255, 0, 255]);
const notFilled = new Uint8Array([0, 0, 0, 0]);

const kRenderTargetFormat = 'rgba8unorm';

class F extends GPUTest {
  MakeIndexBuffer() {
    return this.makeBufferWithContents(
      new Uint32Array([
      0, 1, 2, // The bottom left triangle
      1, 2, 3 // The top right triangle
      ]),
      GPUBufferUsage.INDEX
    );
  }

  MakeVertexBuffer(isIndexed) {

    const vertices = isIndexed ?
    [
    -1.0, -1.0,
    -1.0, 1.0,
    1.0, -1.0,
    1.0, 1.0] :

    [
    // The bottom left triangle
    -1.0, 1.0,
    1.0, -1.0,
    -1.0, -1.0,

    // The top right triangle
    -1.0, 1.0,
    1.0, -1.0,
    1.0, 1.0];

    return this.makeBufferWithContents(new Float32Array(vertices), GPUBufferUsage.VERTEX);
  }

  MakeIndirectBuffer(isIndexed, indirectOffset) {
    const o = indirectOffset / Uint32Array.BYTES_PER_ELEMENT;

    const parametersSize = isIndexed ?
    kDrawIndexedIndirectParametersSize :
    kDrawIndirectParametersSize;
    const arraySize = o + parametersSize * 2;

    const indirectBuffer = [...Array(arraySize)].map(() => Math.floor(Math.random() * 100));

    if (isIndexed) {
      // draw args that will draw the left bottom triangle (expected call)
      indirectBuffer[o] = 3; // indexCount
      indirectBuffer[o + 1] = 1; // instanceCount
      indirectBuffer[o + 2] = 0; // firstIndex
      indirectBuffer[o + 3] = 0; // baseVertex
      indirectBuffer[o + 4] = 0; // firstInstance

      // draw args that will draw both triangles
      indirectBuffer[o + 5] = 6; // indexCount
      indirectBuffer[o + 6] = 1; // instanceCount
      indirectBuffer[o + 7] = 0; // firstIndex
      indirectBuffer[o + 8] = 0; // baseVertex
      indirectBuffer[o + 9] = 0; // firstInstance

      if (o >= parametersSize) {
        // draw args that will draw the right top triangle
        indirectBuffer[o - 5] = 3; // indexCount
        indirectBuffer[o - 4] = 1; // instanceCount
        indirectBuffer[o - 3] = 3; // firstIndex
        indirectBuffer[o - 2] = 0; // baseVertex
        indirectBuffer[o - 1] = 0; // firstInstance
      }

      if (o >= parametersSize * 2) {
        // draw args that will draw nothing
        indirectBuffer[0] = 0; // indexCount
        indirectBuffer[1] = 0; // instanceCount
        indirectBuffer[2] = 0; // firstIndex
        indirectBuffer[3] = 0; // baseVertex
        indirectBuffer[4] = 0; // firstInstance
      }
    } else {
      // draw args that will draw the left bottom triangle (expected call)
      indirectBuffer[o] = 3; // vertexCount
      indirectBuffer[o + 1] = 1; // instanceCount
      indirectBuffer[o + 2] = 0; // firstVertex
      indirectBuffer[o + 3] = 0; // firstInstance

      // draw args that will draw both triangles
      indirectBuffer[o + 4] = 6; // vertexCount
      indirectBuffer[o + 5] = 1; // instanceCount
      indirectBuffer[o + 6] = 0; // firstVertex
      indirectBuffer[o + 7] = 0; // firstInstance

      if (o >= parametersSize) {
        // draw args that will draw the right top triangle
        indirectBuffer[o - 4] = 3; // vertexCount
        indirectBuffer[o - 3] = 1; // instanceCount
        indirectBuffer[o - 2] = 3; // firstVertex
        indirectBuffer[o - 1] = 0; // firstInstance
      }

      if (o >= parametersSize * 2) {
        // draw args that will draw nothing
        indirectBuffer[0] = 0; // vertexCount
        indirectBuffer[1] = 0; // instanceCount
        indirectBuffer[2] = 0; // firstVertex
        indirectBuffer[3] = 0; // firstInstance
      }
    }

    return this.makeBufferWithContents(new Uint32Array(indirectBuffer), GPUBufferUsage.INDIRECT);
  }
}

export const g = makeTestGroup(TextureTestMixin(F));

g.test('basics').
desc(
  `Test that the indirect draw parameters are tightly packed for drawIndirect and drawIndexedIndirect.
An indirectBuffer is created based on indirectOffset. The actual draw args being used indicated by the
indirectOffset is going to draw a left bottom triangle.
While the remaining indirectBuffer is populated with random numbers or draw args
that draw right top triangle, both, or nothing which will fail the color check.
The test will check render target to see if only the left bottom area is filled,
meaning the expected draw args is uploaded correctly by the indirectBuffer and indirectOffset.

Params:
    - draw{Indirect, IndexedIndirect}
    - indirectOffset= {0, 4, k * sizeof(args struct), k * sizeof(args struct) + 4}
    `
).
params((u) =>
u.
combine('isIndexed', [true, false]).
beginSubcases().
expand('indirectOffset', (p) => {
  const indirectDrawParametersSize = p.isIndexed ?
  kDrawIndexedIndirectParametersSize * Uint32Array.BYTES_PER_ELEMENT :
  kDrawIndirectParametersSize * Uint32Array.BYTES_PER_ELEMENT;
  return [
  0,
  Uint32Array.BYTES_PER_ELEMENT,
  1 * indirectDrawParametersSize,
  1 * indirectDrawParametersSize + Uint32Array.BYTES_PER_ELEMENT,
  3 * indirectDrawParametersSize,
  3 * indirectDrawParametersSize + Uint32Array.BYTES_PER_ELEMENT,
  99 * indirectDrawParametersSize,
  99 * indirectDrawParametersSize + Uint32Array.BYTES_PER_ELEMENT];

})
).
fn((t) => {
  const { isIndexed, indirectOffset } = t.params;

  const vertexBuffer = t.MakeVertexBuffer(isIndexed);
  const indirectBuffer = t.MakeIndirectBuffer(isIndexed, indirectOffset);

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: {
      module: t.device.createShaderModule({
        code: `@vertex fn main(@location(0) pos : vec2<f32>) -> @builtin(position) vec4<f32> {
              return vec4<f32>(pos, 0.0, 1.0);
          }`
      }),
      entryPoint: 'main',
      buffers: [
      {
        attributes: [
        {
          shaderLocation: 0,
          format: 'float32x2',
          offset: 0
        }],

        arrayStride: 2 * Float32Array.BYTES_PER_ELEMENT
      }]

    },
    fragment: {
      module: t.device.createShaderModule({
        code: `@fragment fn main() -> @location(0) vec4<f32> {
            return vec4<f32>(0.0, 1.0, 0.0, 1.0);
        }`
      }),
      entryPoint: 'main',
      targets: [
      {
        format: kRenderTargetFormat
      }]

    }
  });

  const renderTarget = t.createTextureTracked({
    size: [4, 4],
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC,
    format: kRenderTargetFormat
  });

  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = commandEncoder.beginRenderPass({
    colorAttachments: [
    {
      view: renderTarget.createView(),
      clearValue: [0, 0, 0, 0],
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  renderPass.setPipeline(pipeline);
  renderPass.setVertexBuffer(0, vertexBuffer, 0);

  if (isIndexed) {
    renderPass.setIndexBuffer(t.MakeIndexBuffer(), 'uint32', 0);
    renderPass.drawIndexedIndirect(indirectBuffer, indirectOffset);
  } else {
    renderPass.drawIndirect(indirectBuffer, indirectOffset);
  }
  renderPass.end();
  t.queue.submit([commandEncoder.finish()]);

  t.expectSinglePixelComparisonsAreOkInTexture({ texture: renderTarget }, [
  // The bottom left area is filled
  { coord: { x: 0, y: 1 }, exp: filled },
  // The top right area is not filled
  { coord: { x: 1, y: 0 }, exp: notFilled }]
  );
});