/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
render pass validation tests.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';

import { ValidationTest } from './validation_test.js';

class F extends ValidationTest {
  getUniformBuffer() {
    return this.device.createBuffer({
      size: 8 * Float32Array.BYTES_PER_ELEMENT,
      usage: GPUBufferUsage.UNIFORM,
    });
  }

  createRenderPipeline(pipelineLayout) {
    const vertexModule = this.makeShaderModule('vertex', {
      glsl: `#version 450
          layout (set = 0, binding = 0) uniform vertexUniformBuffer {
              mat2 transform;
          };
          void main() {
              const vec2 pos[3] = vec2[3](vec2(-1.f, -1.f), vec2(1.f, -1.f), vec2(-1.f, 1.f));
              gl_Position = vec4(transform * pos[gl_VertexIndex], 0.f, 1.f);
          }
        `,
    });

    const fragmentModule = this.makeShaderModule('fragment', {
      glsl: `
        #version 450
        layout (set = 1, binding = 0) uniform fragmentUniformBuffer {
          vec4 color;
        };
        layout(location = 0) out vec4 fragColor;
        void main() {
        }
      `,
    });

    const pipeline = this.device.createRenderPipeline({
      vertexStage: { module: vertexModule, entryPoint: 'main' },
      fragmentStage: { module: fragmentModule, entryPoint: 'main' },
      layout: pipelineLayout,
      primitiveTopology: 'triangle-list',
      colorStates: [{ format: 'rgba8unorm' }],
    });

    return pipeline;
  }

  beginRenderPass(commandEncoder) {
    const attachmentTexture = this.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 16, height: 16, depth: 1 },
      usage: GPUTextureUsage.OUTPUT_ATTACHMENT,
    });

    return commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          attachment: attachmentTexture.createView(),
          loadValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
        },
      ],
    });
  }
}

export const g = makeTestGroup(F);

g.test('it_is_invalid_to_draw_in_a_render_pass_with_missing_bind_groups')
  .params([
    { setBindGroup1: true, setBindGroup2: true, _success: true },
    { setBindGroup1: true, setBindGroup2: false, _success: false },
    { setBindGroup1: false, setBindGroup2: true, _success: false },
    { setBindGroup1: false, setBindGroup2: false, _success: false },
  ])
  .fn(async t => {
    const { setBindGroup1, setBindGroup2, _success } = t.params;

    const uniformBuffer = t.getUniformBuffer();

    const bindGroupLayout1 = t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.VERTEX,
          type: 'uniform-buffer',
        },
      ],
    });

    const bindGroup1 = t.device.createBindGroup({
      entries: [
        {
          binding: 0,
          resource: {
            buffer: uniformBuffer,
          },
        },
      ],

      layout: bindGroupLayout1,
    });

    const bindGroupLayout2 = t.device.createBindGroupLayout({
      entries: [
        {
          binding: 0,
          visibility: GPUShaderStage.FRAGMENT,
          type: 'uniform-buffer',
        },
      ],
    });

    const bindGroup2 = t.device.createBindGroup({
      entries: [
        {
          binding: 0,
          resource: {
            buffer: uniformBuffer,
          },
        },
      ],

      layout: bindGroupLayout2,
    });

    const pipelineLayout = t.device.createPipelineLayout({
      bindGroupLayouts: [bindGroupLayout1, bindGroupLayout2],
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
  });
