/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
render pass validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  getUniformBuffer() {
    return this.device.createBuffer({
      size: 4 * Float32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.UNIFORM
    });
  }

  createRenderPipeline(pipelineLayout) {
    const vertexModule = this.createShaderModule({
      code:
      /* GLSL(
       *         'vertex',
       *         `#version 450
       *           layout (set = 0, binding = 0) uniform vertexUniformBuffer {
       *               mat2 transform;
       *           };
       *           void main() {
       *               const vec2 pos[3] = vec2[3](vec2(-1.f, -1.f), vec2(1.f, -1.f), vec2(-1.f, 1.f));
       *               gl_Position = vec4(transform * pos[gl_VertexIndex], 0.f, 1.f);
       *           }
       *         `
       *       )
       */
      new Uint32Array([119734787, 66304, 524295, 47, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 458767, 0, 4, 1852399981, 0, 13, 33, 196611, 2, 450, 262149, 4, 1852399981, 0, 393221, 11, 1348430951, 1700164197, 2019914866, 0, 393222, 11, 0, 1348430951, 1953067887, 7237481, 458758, 11, 1, 1348430951, 1953393007, 1702521171, 0, 458758, 11, 2, 1130327143, 1148217708, 1635021673, 6644590, 458758, 11, 3, 1130327143, 1147956341, 1635021673, 6644590, 196613, 13, 0, 458757, 18, 1953654134, 1851095141, 1919903337, 1718960749, 7497062, 393222, 18, 0, 1851880052, 1919903347, 109, 196613, 20, 0, 393221, 33, 1449094247, 1702130277, 1684949368, 30821, 327685, 36, 1701080681, 1818386808, 101, 327752, 11, 0, 11, 0, 327752, 11, 1, 11, 1, 327752, 11, 2, 11, 3, 327752, 11, 3, 11, 4, 196679, 11, 2, 262216, 18, 0, 5, 327752, 18, 0, 35, 0, 327752, 18, 0, 7, 16, 196679, 18, 2, 262215, 20, 34, 0, 262215, 20, 33, 0, 262215, 33, 11, 42, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 262165, 8, 32, 0, 262187, 8, 9, 1, 262172, 10, 6, 9, 393246, 11, 7, 6, 10, 10, 262176, 12, 3, 11, 262203, 12, 13, 3, 262165, 14, 32, 1, 262187, 14, 15, 0, 262167, 16, 6, 2, 262168, 17, 16, 2, 196638, 18, 17, 262176, 19, 2, 18, 262203, 19, 20, 2, 262176, 21, 2, 17, 262187, 8, 24, 3, 262172, 25, 16, 24, 262187, 6, 26, 3212836864, 327724, 16, 27, 26, 26, 262187, 6, 28, 1065353216, 327724, 16, 29, 28, 26, 327724, 16, 30, 26, 28, 393260, 25, 31, 27, 29, 30, 262176, 32, 1, 14, 262203, 32, 33, 1, 262176, 35, 7, 25, 262176, 37, 7, 16, 262187, 6, 41, 0, 262176, 45, 3, 7, 327734, 2, 4, 0, 3, 131320, 5, 262203, 35, 36, 7, 327745, 21, 22, 20, 15, 262205, 17, 23, 22, 262205, 14, 34, 33, 196670, 36, 31, 327745, 37, 38, 36, 34, 262205, 16, 39, 38, 327825, 16, 40, 23, 39, 327761, 6, 42, 40, 0, 327761, 6, 43, 40, 1, 458832, 7, 44, 42, 43, 41, 28, 327745, 45, 46, 13, 15, 196670, 46, 44, 65789, 65592])
    });
    const fragmentModule = this.createShaderModule({
      code:
      /* GLSL(
       *         'fragment',
       *         `#version 450
       *           layout (set = 1, binding = 0) uniform fragmentUniformBuffer {
       *             vec4 color;
       *           };
       *           layout(location = 0) out vec4 fragColor;
       *           void main() {
       *           }
       *         `
       *       )
       */
      new Uint32Array([119734787, 66304, 524295, 13, 0, 131089, 1, 393227, 1, 1280527431, 1685353262, 808793134, 0, 196622, 0, 1, 393231, 4, 4, 1852399981, 0, 12, 196624, 4, 7, 196611, 2, 450, 262149, 4, 1852399981, 0, 524293, 8, 1734439526, 1953391981, 1718185557, 1114468975, 1701209717, 114, 327686, 8, 0, 1869377379, 114, 196613, 10, 0, 327685, 12, 1734439526, 1869377347, 114, 327752, 8, 0, 35, 0, 196679, 8, 2, 262215, 10, 34, 1, 262215, 10, 33, 0, 262215, 12, 30, 0, 131091, 2, 196641, 3, 2, 196630, 6, 32, 262167, 7, 6, 4, 196638, 8, 7, 262176, 9, 2, 8, 262203, 9, 10, 2, 262176, 11, 3, 7, 262203, 11, 12, 3, 327734, 2, 4, 0, 3, 131320, 5, 65789, 65592])
    });
    const pipeline = this.device.createRenderPipeline({
      vertexStage: {
        module: vertexModule,
        entryPoint: 'main'
      },
      fragmentStage: {
        module: fragmentModule,
        entryPoint: 'main'
      },
      layout: pipelineLayout,
      primitiveTopology: 'triangle-list',
      colorStates: [{
        format: 'rgba8unorm'
      }]
    });
    return pipeline;
  }

  beginRenderPass(commandEncoder) {
    const attachmentTexture = this.device.createTexture({
      format: 'rgba8unorm',
      size: {
        width: 16,
        height: 16,
        depth: 1
      },
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT
    });
    return commandEncoder.beginRenderPass({
      colorAttachments: [{
        attachment: attachmentTexture.createView(),
        loadValue: {
          r: 1.0,
          g: 0.0,
          b: 0.0,
          a: 1.0
        }
      }]
    });
  }

}

export const g = new TestGroup(F);
g.test('it is invalid to draw in a render pass with missing bind groups', async t => {
  const {
    setBindGroup1,
    setBindGroup2,
    _success
  } = t.params;
  const uniformBuffer = t.getUniformBuffer();
  const bindGroupLayout1 = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.VERTEX,
      type: 'uniform-buffer'
    }]
  });
  const bindGroup1 = t.device.createBindGroup({
    bindings: [{
      binding: 0,
      resource: {
        buffer: uniformBuffer
      }
    }],
    layout: bindGroupLayout1
  });
  const bindGroupLayout2 = t.device.createBindGroupLayout({
    bindings: [{
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT,
      type: 'uniform-buffer'
    }]
  });
  const bindGroup2 = t.device.createBindGroup({
    bindings: [{
      binding: 0,
      resource: {
        buffer: uniformBuffer
      }
    }],
    layout: bindGroupLayout2
  });
  const pipelineLayout = t.device.createPipelineLayout({
    bindGroupLayouts: [bindGroupLayout1, bindGroupLayout2]
  });
  const pipeline = t.createRenderPipeline(pipelineLayout);
  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = t.beginRenderPass(commandEncoder);
  renderPass.setPipeline(pipeline);

  if (setBindGroup1) {
    renderPass.setBindGroup(0, bindGroup1);
  }

  if (setBindGroup2) {
    renderPass.setBindGroup(1, bindGroup2);
  }

  renderPass.draw(3, 1, 0, 0);
  renderPass.endPass();
  t.expectValidationError(() => {
    commandEncoder.finish();
  }, !_success);
}).params([{
  setBindGroup1: true,
  setBindGroup2: true,
  _success: true
}, {
  setBindGroup1: true,
  setBindGroup2: false,
  _success: false
}, {
  setBindGroup1: false,
  setBindGroup2: true,
  _success: false
}, {
  setBindGroup1: false,
  setBindGroup2: false,
  _success: false
}]);
//# sourceMappingURL=render_pass.spec.js.map