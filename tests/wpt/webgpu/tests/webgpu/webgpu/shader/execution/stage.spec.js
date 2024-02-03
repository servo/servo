/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Test trivial shaders for each shader stage kind`; // There are many many more shaders executed in other tests.

import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUTest } from '../../gpu_test.js';
import { checkElementsEqual } from '../../util/check_contents.js';

export const g = makeTestGroup(GPUTest);

g.test('basic_compute').
desc(`Test a trivial compute shader`).
fn(async (t) => {
  const code = `

@group(0) @binding(0)
var<storage, read_write> v : vec4u;

@compute @workgroup_size(1)
fn main() {
  v = vec4u(1,2,3,42);
}`;

  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({
        code
      }),
      entryPoint: 'main'
    }
  });

  const buffer = t.makeBufferWithContents(
    new Uint32Array([0, 0, 0, 0]),
    GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  );
  t.trackForCleanup(buffer);

  const bg = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    {
      binding: 0,
      resource: {
        buffer
      }
    }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bg);
  pass.dispatchWorkgroups(1, 1, 1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  const bufferReadback = await t.readGPUBufferRangeTyped(buffer, {
    srcByteOffset: 0,
    type: Uint32Array,
    typedLength: 4,
    method: 'copy'
  });
  const got = bufferReadback.data;
  const expected = new Uint32Array([1, 2, 3, 42]);

  t.expectOK(checkElementsEqual(got, expected));
});

g.test('basic_render').
desc(`Test trivial vertex and fragment shaders`).
fn((t) => {
  const code = `
@vertex
fn vert_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4f {
  // A right triangle covering the whole framebuffer.
  const pos = array(
    vec2f(-1,-3),
    vec2f(3,1),
    vec2f(-1,1));
  return vec4f(pos[idx], 0, 1);
}

@fragment
fn frag_main() -> @location(0) vec4f {
  return vec4(0, 1, 0, 1); // green
}
`;
  const module = t.device.createShaderModule({ code });

  const [width, height] = [8, 8];
  const format = 'rgba8unorm';
  const texture = t.device.createTexture({
    size: { width, height },
    usage:
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.COPY_SRC,
    format
  });

  // We'll copy one pixel only.
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });

  const pipeline = t.device.createRenderPipeline({
    layout: 'auto',
    vertex: { module, entryPoint: 'vert_main' },
    fragment: { module, entryPoint: 'frag_main', targets: [{ format }] }
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [{ view: texture.createView(), loadOp: 'clear', storeOp: 'store' }]
  });
  pass.setPipeline(pipeline);
  pass.draw(3);
  pass.end();

  encoder.copyTextureToBuffer(
    { texture, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { buffer: dst, bytesPerRow: 256 },
    { width: 1, height: 1, depthOrArrayLayers: 1 }
  );
  t.queue.submit([encoder.finish()]);

  // Expect one green pixel.
  t.expectGPUBufferValuesEqual(dst, new Uint8Array([0x00, 0xff, 0x00, 0xff]));
});