/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
Basic command buffer compute tests.
`;
import { TestGroup } from '../../../../framework/index.js';
import { GPUTest } from '../../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('memcpy', async t => {
  const data = new Uint32Array([0x01020304]);
  const src = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE
  });
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE
  });
  src.setSubData(0, data);
  const bgl = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: 4,
      type: 'storage-buffer'
    }, {
      binding: 1,
      visibility: 4,
      type: 'storage-buffer'
    }]
  });
  const bg = t.device.createBindGroup({
    bindings: [{
      binding: 0,
      resource: {
        buffer: src,
        offset: 0,
        size: 4
      }
    }, {
      binding: 1,
      resource: {
        buffer: dst,
        offset: 0,
        size: 4
      }
    }],
    layout: bgl
  });
  const module = t.createShaderModule({
    code:
    /* GLSL(
     *       'compute',
     *       `#version 310 es
     *         layout(std140, set = 0, binding = 0) buffer Src {
     *           int value;
     *         } src;
     *         layout(std140, set = 0, binding = 1) buffer Dst {
     *           int value;
     *         } dst;
     *
     *         void main() {
     *           dst.value = src.value;
     *         }
     *       `
     *     )
     */
    new Uint32Array([119734787, 66304, 524295, 18, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 327695, 5, 4, 1852399981, 0, 393232, 4, 17, 1, 1, 1, 196611, 1, 310, 262149, 4, 1852399981, 0, 196613, 7, 7631684, 327686, 7, 0, 1970037110, 101, 196613, 9, 7631716, 196613, 11, 6517331, 327686, 11, 0, 1970037110, 101, 196613, 13, 6517363, 327752, 7, 0, 35, 0, 196679, 7, 2, 262215, 9, 34, 0, 262215, 9, 33, 1, 327752, 11, 0, 35, 0, 196679, 11, 2, 262215, 13, 34, 0, 262215, 13, 33, 0, 131091, 2, 196641, 3, 2, 262165, 6, 32, 1, 196638, 7, 6, 262176, 8, 12, 7, 262203, 8, 9, 12, 262187, 6, 10, 0, 196638, 11, 6, 262176, 12, 12, 11, 262203, 12, 13, 12, 262176, 14, 12, 6, 327734, 2, 4, 0, 3, 131320, 5, 327745, 14, 15, 13, 10, 262205, 6, 16, 15, 327745, 14, 17, 9, 10, 196670, 17, 16, 65789, 65592])
  });
  const pl = t.device.createPipelineLayout({
    bindGroupLayouts: [bgl]
  });
  const pipeline = t.device.createComputePipeline({
    computeStage: {
      module,
      entryPoint: 'main'
    },
    layout: pl
  });
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatch(1, 1, 1);
  pass.endPass();
  t.device.defaultQueue.submit([encoder.finish()]);
  t.expectContents(dst, data);
});
//# sourceMappingURL=basic.spec.js.map