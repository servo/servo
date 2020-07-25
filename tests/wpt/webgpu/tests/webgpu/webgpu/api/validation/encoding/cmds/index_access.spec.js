/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
indexed draws validation tests.
`;
import { params, poptions, pbool } from '../../../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../../../common/framework/test_group.js';

import { ValidationTest } from './../../validation_test.js';

class F extends ValidationTest {
  createIndexBuffer() {
    const indexArray = new Uint32Array([0, 1, 2, 3, 1, 2]);

    const [indexBuffer, indexMapping] = this.device.createBufferMapped({
      size: indexArray.byteLength,
      usage: GPUBufferUsage.INDEX,
    });

    new Uint32Array(indexMapping).set(indexArray);
    indexBuffer.unmap();

    return indexBuffer;
  }

  createRenderPipeline() {
    const vertexModule = this.makeShaderModule('vertex', {
      glsl: `
        #version 450
        void main() {
          gl_Position = vec4(0.0, 0.0, 0.0, 1.0);
        }
      `,
    });

    const fragmentModule = this.makeShaderModule('fragment', {
      glsl: `
        #version 450
        layout(location = 0) out vec4 fragColor;
        void main() {
            fragColor = vec4(0.0, 1.0, 0.0, 1.0);
        }
      `,
    });

    return this.device.createRenderPipeline({
      layout: this.device.createPipelineLayout({ bindGroupLayouts: [] }),
      vertexStage: { module: vertexModule, entryPoint: 'main' },
      fragmentStage: { module: fragmentModule, entryPoint: 'main' },
      primitiveTopology: 'triangle-strip',
      colorStates: [{ format: 'rgba8unorm' }],
    });
  }

  beginRenderPass(encoder) {
    const colorAttachment = this.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 1, height: 1, depth: 1 },
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    return encoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: colorAttachment.createView(),
          loadValue: { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
          storeOp: 'store',
        },
      ],
    });
  }

  drawIndexed(indexCount, instanceCount, firstIndex, baseVertex, firstInstance) {
    const indexBuffer = this.createIndexBuffer();

    const pipeline = this.createRenderPipeline();

    const encoder = this.device.createCommandEncoder();
    const pass = this.beginRenderPass(encoder);
    pass.setPipeline(pipeline);
    pass.setIndexBuffer(indexBuffer);
    pass.drawIndexed(indexCount, instanceCount, firstIndex, baseVertex, firstInstance);
    pass.endPass();

    this.device.defaultQueue.submit([encoder.finish()]);
  }

  drawIndexedIndirect(bufferArray, indirectOffset) {
    const [indirectBuffer, indirectMapping] = this.device.createBufferMapped({
      size: bufferArray.byteLength,
      usage: GPUBufferUsage.INDIRECT,
    });

    new Uint32Array(indirectMapping).set(bufferArray);
    indirectBuffer.unmap();

    const indexBuffer = this.createIndexBuffer();

    const pipeline = this.createRenderPipeline();

    const encoder = this.device.createCommandEncoder();
    const pass = this.beginRenderPass(encoder);
    pass.setPipeline(pipeline);
    pass.setIndexBuffer(indexBuffer, 0);
    pass.drawIndexedIndirect(indirectBuffer, indirectOffset);
    pass.endPass();

    this.device.defaultQueue.submit([encoder.finish()]);
  }
}

export const g = makeTestGroup(F);

g.test('out_of_bounds')
  .params(
    params()
      .combine(pbool('indirect')) // indirect drawIndexed
      .combine([
        { indexCount: 6, firstIndex: 1 }, // indexCount + firstIndex out of bound
        { indexCount: 6, firstIndex: 6 }, // only firstIndex out of bound
        { indexCount: 6, firstIndex: 10000 }, // firstIndex much larger than the bound
        { indexCount: 7, firstIndex: 0 }, // only indexCount out of bound
        { indexCount: 10000, firstIndex: 0 }, // indexCount much larger than the bound
      ])
      .combine(poptions('instanceCount', [1, 10000])) // normal and large instanceCount
  )
  .fn(t => {
    const { indirect, indexCount, firstIndex, instanceCount } = t.params;

    if (indirect) {
      t.drawIndexedIndirect(new Uint32Array([indexCount, instanceCount, firstIndex, 0, 0]), 0);
    } else {
      t.drawIndexed(indexCount, instanceCount, firstIndex, 0, 0);
    }
  });
