/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = ``;
import { TestGroup } from '../../../../framework/index.js';
import { GPUTest } from '../../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('fullscreen quad', async t => {
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST
  });
  const colorAttachment = t.device.createTexture({
    format: 'rgba8unorm',
    size: {
      width: 1,
      height: 1,
      depth: 1
    },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT
  });
  const colorAttachmentView = colorAttachment.createDefaultView();
  const vertexModule = t.device.createShaderModule({
    code:
    /* GLSL(
     *       'vertex',
     *       `#version 310 es
     *         void main() {
     *           const vec2 pos[3] = vec2[3](
     *               vec2(-1.f, -3.f), vec2(3.f, 1.f), vec2(-1.f, 1.f));
     *           gl_Position = vec4(pos[gl_VertexIndex], 0.f, 1.f);
     *         }
     *       `
     *     )
     */
    new Uint32Array([119734787, 66304, 524295, 39, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 458767, 0, 4, 1852399981, 0, 10, 26, 196611, 1, 310, 262149, 4, 1852399981, 0, 393221, 8, 1348430951, 1700164197, 2019914866, 0, 393222, 8, 0, 1348430951, 1953067887, 7237481, 458758, 8, 1, 1348430951, 1953393007, 1702521171, 0, 196613, 10, 0, 393221, 26, 1449094247, 1702130277, 1684949368, 30821, 327685, 29, 1701080681, 1818386808, 101, 327752, 8, 0, 11, 0, 327752, 8, 1, 11, 1, 196679, 8, 2, 262215, 26, 11, 42, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 262174, 8, 7, 6, 262176, 9, 3, 8, 262203, 9, 10, 3, 262165, 11, 32, 1, 262187, 11, 12, 0, 262167, 13, 6, 2, 262165, 14, 32, 0, 262187, 14, 15, 3, 262172, 16, 13, 15, 262187, 6, 17, 3212836864, 262187, 6, 18, 3225419776, 327724, 13, 19, 17, 18, 262187, 6, 20, 1077936128, 262187, 6, 21, 1065353216, 327724, 13, 22, 20, 21, 327724, 13, 23, 17, 21, 393260, 16, 24, 19, 22, 23, 262176, 25, 1, 11, 262203, 25, 26, 1, 262176, 28, 7, 16, 262176, 30, 7, 13, 262187, 6, 33, 0, 262176, 37, 3, 7, 327734, 2, 4, 0, 3, 131320, 5, 262203, 28, 29, 7, 262205, 11, 27, 26, 196670, 29, 24, 327745, 30, 31, 29, 27, 262205, 13, 32, 31, 327761, 6, 34, 32, 0, 327761, 6, 35, 32, 1, 458832, 7, 36, 34, 35, 33, 21, 327745, 37, 38, 10, 12, 196670, 38, 36, 65789, 65592])
  });
  const fragmentModule = t.device.createShaderModule({
    code:
    /* GLSL(
     *       'fragment',
     *       `#version 310 es
     *         precision mediump float;
     *         layout(location = 0) out vec4 fragColor;
     *         void main() {
     *           fragColor = vec4(0.0, 1.0, 0.0, 1.0);
     *         }
     *       `
     *     )
     */
    new Uint32Array([119734787, 66304, 524295, 13, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 393231, 4, 4, 1852399981, 0, 9, 196624, 4, 7, 196611, 1, 310, 262149, 4, 1852399981, 0, 327685, 9, 1734439526, 1869377347, 114, 196679, 9, 0, 262215, 9, 30, 0, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 262176, 8, 3, 7, 262203, 8, 9, 3, 262187, 6, 10, 0, 262187, 6, 11, 1065353216, 458796, 7, 12, 10, 11, 10, 11, 327734, 2, 4, 0, 3, 131320, 5, 196670, 9, 12, 65789, 65592])
  });
  const pl = t.device.createPipelineLayout({
    bindGroupLayouts: []
  });
  const pipeline = t.device.createRenderPipeline({
    vertexStage: {
      module: vertexModule,
      entryPoint: 'main'
    },
    fragmentStage: {
      module: fragmentModule,
      entryPoint: 'main'
    },
    layout: pl,
    primitiveTopology: 'triangle-list',
    rasterizationState: {
      frontFace: 'ccw'
    },
    colorStates: [{
      format: 'rgba8unorm',
      alphaBlend: {},
      colorBlend: {}
    }],
    vertexInput: {
      indexFormat: 'uint16',
      vertexBuffers: []
    }
  });
  const encoder = t.device.createCommandEncoder({});
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      attachment: colorAttachmentView,
      storeOp: 'store',
      loadValue: {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0
      }
    }]
  });
  pass.setPipeline(pipeline);
  pass.draw(3, 1, 0, 0);
  pass.endPass();
  encoder.copyTextureToBuffer({
    texture: colorAttachment,
    mipLevel: 0,
    origin: {
      x: 0,
      y: 0,
      z: 0
    }
  }, {
    buffer: dst,
    rowPitch: 256,
    imageHeight: 1
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  t.device.getQueue().submit([encoder.finish()]);
  await t.expectContents(dst, new Uint8Array([0x00, 0xff, 0x00, 0xff]));
});
//# sourceMappingURL=rendering.spec.js.map