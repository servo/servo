/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Basic command buffer compute tests.
`;
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('memcpy').fn(async t => {
  const data = new Uint32Array([0x01020304]);

  const [src, srcData] = t.device.createBufferMapped({
    size: 4,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.STORAGE,
  });

  new Uint32Array(srcData).set(data);
  src.unmap();

  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE,
  });

  const bgl = t.device.createBindGroupLayout({
    entries: [
      { binding: 0, visibility: 4, type: 'storage-buffer' },
      { binding: 1, visibility: 4, type: 'storage-buffer' },
    ],
  });

  const bg = t.device.createBindGroup({
    entries: [
      { binding: 0, resource: { buffer: src, offset: 0, size: 4 } },
      { binding: 1, resource: { buffer: dst, offset: 0, size: 4 } },
    ],

    layout: bgl,
  });

  const module = t.makeShaderModule('compute', {
    glsl: `
      #version 310 es
      layout(std140, set = 0, binding = 0) buffer Src {
        int value;
      } src;
      layout(std140, set = 0, binding = 1) buffer Dst {
        int value;
      } dst;

      void main() {
        dst.value = src.value;
      }
    `,
  });

  const pl = t.device.createPipelineLayout({ bindGroupLayouts: [bgl] });
  const pipeline = t.device.createComputePipeline({
    computeStage: { module, entryPoint: 'main' },
    layout: pl,
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
