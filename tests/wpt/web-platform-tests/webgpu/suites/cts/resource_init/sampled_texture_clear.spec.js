/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
computePass test that sampled texture is cleared`;
import { TestGroup } from '../../../framework/index.js';
import { GPUTest } from '../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('compute pass test that sampled texture is cleared', async t => {
  const texture = t.device.createTexture({
    size: {
      width: 256,
      height: 256,
      depth: 1
    },
    format: 'r8unorm',
    usage: GPUTextureUsage.SAMPLED
  });
  const bufferTex = t.device.createBuffer({
    size: 4 * 256 * 256,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST
  });
  const sampler = t.device.createSampler();
  const bindGroupLayout = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      type: 'sampled-texture'
    }, {
      binding: 1,
      visibility: GPUShaderStage.COMPUTE,
      type: 'storage-buffer'
    }, {
      binding: 2,
      visibility: GPUShaderStage.COMPUTE,
      type: 'sampler'
    }]
  }); // create compute pipeline

  const computeModule = t.device.createShaderModule({
    code:
    /* GLSL(
     *       'compute',
     *       `#version 450
     *       layout(binding = 0) uniform texture2D sampleTex;
     *       layout(std430, binding = 1) buffer BufferTex {
     *          vec4 result;
     *       } bufferTex;
     *       layout(binding = 2) uniform sampler sampler0;
     *       void main() {
     *          bufferTex.result =
     *                texelFetch(sampler2D(sampleTex, sampler0), ivec2(0,0), 0);
     *       }`
     *     )
     */
    new Uint32Array([119734787, 66304, 524296, 29, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 327695, 5, 4, 1852399981, 0, 393232, 4, 17, 1, 1, 1, 196611, 2, 450, 262149, 4, 1852399981, 0, 327685, 8, 1717990722, 1700033125, 120, 327686, 8, 0, 1970496882, 29804, 327685, 10, 1717990754, 1700033125, 120, 327685, 15, 1886216563, 1700029804, 120, 327685, 19, 1886216563, 812803436, 0, 327752, 8, 0, 35, 0, 196679, 8, 2, 262215, 10, 34, 0, 262215, 10, 33, 1, 262215, 15, 34, 0, 262215, 15, 33, 0, 262215, 19, 34, 0, 262215, 19, 33, 2, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 196638, 8, 7, 262176, 9, 12, 8, 262203, 9, 10, 12, 262165, 11, 32, 1, 262187, 11, 12, 0, 589849, 13, 6, 1, 0, 0, 0, 1, 0, 262176, 14, 0, 13, 262203, 14, 15, 0, 131098, 17, 262176, 18, 0, 17, 262203, 18, 19, 0, 196635, 21, 13, 262167, 23, 11, 2, 327724, 23, 24, 12, 12, 262176, 27, 12, 7, 327734, 2, 4, 0, 3, 131320, 5, 262205, 13, 16, 15, 262205, 17, 20, 19, 327766, 21, 22, 16, 20, 262244, 13, 25, 22, 458847, 7, 26, 25, 24, 2, 12, 327745, 27, 28, 10, 12, 196670, 28, 26, 65789, 65592])
  });
  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts: [bindGroupLayout]
  });
  const computePipeline = t.device.createComputePipeline({
    computeStage: {
      module: computeModule,
      entryPoint: 'main'
    },
    layout: pipelineLayout
  }); // create bindgroup

  const bindGroup = t.device.createBindGroup({
    layout: bindGroupLayout,
    bindings: [{
      binding: 0,
      resource: texture.createView()
    }, {
      binding: 1,
      resource: {
        buffer: bufferTex,
        offset: 0,
        size: 4 * 256 * 256
      }
    }, {
      binding: 2,
      resource: sampler
    }]
  }); // encode the pass and submit

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(computePipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatch(256, 256, 1);
  pass.endPass();
  const commands = encoder.finish();
  t.device.defaultQueue.submit([commands]);
  await t.expectContents(bufferTex, new Uint32Array([0]));
});
//# sourceMappingURL=sampled_texture_clear.spec.js.map