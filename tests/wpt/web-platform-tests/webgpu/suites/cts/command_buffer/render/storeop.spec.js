/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
renderPass store op test that drawn quad is either stored or cleared based on storeop`;
import { TestGroup } from '../../../../framework/index.js';
import { GPUTest } from '../../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('storeOp controls whether 1x1 drawn quad is stored', async t => {
  const renderTexture = t.device.createTexture({
    size: {
      width: 1,
      height: 1,
      depth: 1
    },
    format: 'r8unorm',
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT
  }); // create render pipeline

  const vertexModule = t.createShaderModule({
    code:
    /* GLSL(
     *       'vertex',
     *       `#version 450
     *       const vec2 pos[3] = vec2[3](
     *                               vec2( 1.0f, -1.0f),
     *                               vec2( 1.0f,  1.0f),
     *                               vec2(-1.0f,  1.0f)
     *                               );
     *
     *       void main() {
     *           gl_Position = vec4(pos[gl_VertexIndex], 0.0, 1.0);
     *       }`
     *     )
     */
    new Uint32Array([119734787, 66304, 524295, 39, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 458767, 0, 4, 1852399981, 0, 13, 26, 196611, 2, 450, 262149, 4, 1852399981, 0, 393221, 11, 1348430951, 1700164197, 2019914866, 0, 393222, 11, 0, 1348430951, 1953067887, 7237481, 458758, 11, 1, 1348430951, 1953393007, 1702521171, 0, 458758, 11, 2, 1130327143, 1148217708, 1635021673, 6644590, 458758, 11, 3, 1130327143, 1147956341, 1635021673, 6644590, 196613, 13, 0, 393221, 26, 1449094247, 1702130277, 1684949368, 30821, 327685, 29, 1701080681, 1818386808, 101, 327752, 11, 0, 11, 0, 327752, 11, 1, 11, 1, 327752, 11, 2, 11, 3, 327752, 11, 3, 11, 4, 196679, 11, 2, 262215, 26, 11, 42, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 262165, 8, 32, 0, 262187, 8, 9, 1, 262172, 10, 6, 9, 393246, 11, 7, 6, 10, 10, 262176, 12, 3, 11, 262203, 12, 13, 3, 262165, 14, 32, 1, 262187, 14, 15, 0, 262167, 16, 6, 2, 262187, 8, 17, 3, 262172, 18, 16, 17, 262187, 6, 19, 1065353216, 262187, 6, 20, 3212836864, 327724, 16, 21, 19, 20, 327724, 16, 22, 19, 19, 327724, 16, 23, 20, 19, 393260, 18, 24, 21, 22, 23, 262176, 25, 1, 14, 262203, 25, 26, 1, 262176, 28, 7, 18, 262176, 30, 7, 16, 262187, 6, 33, 0, 262176, 37, 3, 7, 327734, 2, 4, 0, 3, 131320, 5, 262203, 28, 29, 7, 262205, 14, 27, 26, 196670, 29, 24, 327745, 30, 31, 29, 27, 262205, 16, 32, 31, 327761, 6, 34, 32, 0, 327761, 6, 35, 32, 1, 458832, 7, 36, 34, 35, 33, 19, 327745, 37, 38, 13, 15, 196670, 38, 36, 65789, 65592])
  });
  const fragmentModule = t.createShaderModule({
    code:
    /* GLSL(
     *       'fragment',
     *       `#version 450
     *       layout(location = 0) out vec4 fragColor;
     *       void main() {
     *           fragColor = vec4(1.0, 0.0, 0.0, 1.0);
     *       }`
     *     )
     */
    new Uint32Array([119734787, 66304, 524295, 13, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 393231, 4, 4, 1852399981, 0, 9, 196624, 4, 7, 196611, 2, 450, 262149, 4, 1852399981, 0, 327685, 9, 1734439526, 1869377347, 114, 262215, 9, 30, 0, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 262176, 8, 3, 7, 262203, 8, 9, 3, 262187, 6, 10, 1065353216, 262187, 6, 11, 0, 458796, 7, 12, 10, 11, 11, 10, 327734, 2, 4, 0, 3, 131320, 5, 196670, 9, 12, 65789, 65592])
  });
  const renderPipeline = t.device.createRenderPipeline({
    vertexStage: {
      module: vertexModule,
      entryPoint: 'main'
    },
    fragmentStage: {
      module: fragmentModule,
      entryPoint: 'main'
    },
    layout: t.device.createPipelineLayout({
      bindGroupLayouts: []
    }),
    primitiveTopology: 'triangle-list',
    colorStates: [{
      format: 'r8unorm'
    }]
  }); // encode pass and submit

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      attachment: renderTexture.createView(),
      storeOp: t.params.storeOp,
      loadValue: {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0
      }
    }]
  });
  pass.setPipeline(renderPipeline);
  pass.draw(3, 1, 0, 0);
  pass.endPass();
  const dstBuffer = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_DST | GPUBufferUsage.COPY_SRC
  });
  encoder.copyTextureToBuffer({
    texture: renderTexture
  }, {
    buffer: dstBuffer,
    rowPitch: 256,
    imageHeight: 1
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  t.device.defaultQueue.submit([encoder.finish()]); // expect the buffer to be clear

  const expectedContent = new Uint32Array([t.params._expected]);
  t.expectContents(dstBuffer, expectedContent);
}).params([{
  storeOp: 'store',
  _expected: 255
}, //
{
  storeOp: 'clear',
  _expected: 0
}]);
//# sourceMappingURL=storeop.spec.js.map