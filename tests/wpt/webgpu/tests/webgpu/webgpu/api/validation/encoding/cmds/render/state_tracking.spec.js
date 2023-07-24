/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Validation tests for setVertexBuffer/setIndexBuffer state (not validation). See also operation tests.
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { range } from '../../../../../../common/util/util.js';
import { ValidationTest } from '../../../validation_test.js';

class F extends ValidationTest {
  getVertexBuffer() {
    return this.device.createBuffer({
      size: 256,
      usage: GPUBufferUsage.VERTEX,
    });
  }

  createRenderPipeline(bufferCount) {
    return this.device.createRenderPipeline({
      layout: 'auto',
      vertex: {
        module: this.device.createShaderModule({
          code: `
            struct Inputs {
            ${range(bufferCount, i => `\n@location(${i}) a_position${i} : vec3<f32>,`).join('')}
            };
            @vertex fn main(input : Inputs
              ) -> @builtin(position) vec4<f32> {
              return vec4<f32>(0.0, 0.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        buffers: [
          {
            arrayStride: 3 * 4,
            attributes: range(bufferCount, i => ({
              format: 'float32x3',
              offset: 0,
              shaderLocation: i,
            })),
          },
        ],
      },
      fragment: {
        module: this.device.createShaderModule({
          code: `
            @fragment fn main() -> @location(0) vec4<f32> {
              return vec4<f32>(0.0, 1.0, 0.0, 1.0);
            }`,
        }),
        entryPoint: 'main',
        targets: [{ format: 'rgba8unorm' }],
      },
      primitive: { topology: 'triangle-list' },
    });
  }

  beginRenderPass(commandEncoder) {
    const attachmentTexture = this.device.createTexture({
      format: 'rgba8unorm',
      size: { width: 16, height: 16, depthOrArrayLayers: 1 },
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
    });

    return commandEncoder.beginRenderPass({
      colorAttachments: [
        {
          view: attachmentTexture.createView(),
          clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
          loadOp: 'clear',
          storeOp: 'store',
        },
      ],
    });
  }
}

export const g = makeTestGroup(F);

g.test(`all_needed_vertex_buffer_should_be_bound`)
  .desc(
    `
In this test we test that any missing vertex buffer for a used slot will cause validation errors when drawing.
- All (non/indexed, in/direct) draw commands
    - A needed vertex buffer is not bound
        - Was bound in another render pass but not the current one
`
  )
  .unimplemented();

g.test(`all_needed_index_buffer_should_be_bound`)
  .desc(
    `
In this test we test that missing index buffer for a used slot will cause validation errors when drawing.
- All indexed in/direct draw commands
    - No index buffer is bound
`
  )
  .unimplemented();

g.test('vertex_buffers_inherit_from_previous_pipeline').fn(t => {
  const pipeline1 = t.createRenderPipeline(1);
  const pipeline2 = t.createRenderPipeline(2);

  const vertexBuffer1 = t.getVertexBuffer();
  const vertexBuffer2 = t.getVertexBuffer();

  {
    // Check failure when vertex buffer is not set
    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = t.beginRenderPass(commandEncoder);
    renderPass.setPipeline(pipeline1);
    renderPass.draw(3);
    renderPass.end();

    t.expectValidationError(() => {
      commandEncoder.finish();
    });
  }
  {
    // Check success when vertex buffer is inherited from previous pipeline
    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = t.beginRenderPass(commandEncoder);
    renderPass.setPipeline(pipeline2);
    renderPass.setVertexBuffer(0, vertexBuffer1);
    renderPass.setVertexBuffer(1, vertexBuffer2);
    renderPass.draw(3);
    renderPass.setPipeline(pipeline1);
    renderPass.draw(3);
    renderPass.end();

    commandEncoder.finish();
  }
});

g.test('vertex_buffers_do_not_inherit_between_render_passes').fn(t => {
  const pipeline1 = t.createRenderPipeline(1);
  const pipeline2 = t.createRenderPipeline(2);

  const vertexBuffer1 = t.getVertexBuffer();
  const vertexBuffer2 = t.getVertexBuffer();

  {
    // Check success when vertex buffer is set for each render pass
    const commandEncoder = t.device.createCommandEncoder();
    {
      const renderPass = t.beginRenderPass(commandEncoder);
      renderPass.setPipeline(pipeline2);
      renderPass.setVertexBuffer(0, vertexBuffer1);
      renderPass.setVertexBuffer(1, vertexBuffer2);
      renderPass.draw(3);
      renderPass.end();
    }
    {
      const renderPass = t.beginRenderPass(commandEncoder);
      renderPass.setPipeline(pipeline1);
      renderPass.setVertexBuffer(0, vertexBuffer1);
      renderPass.draw(3);
      renderPass.end();
    }
    commandEncoder.finish();
  }
  {
    // Check failure because vertex buffer is not inherited in second subpass
    const commandEncoder = t.device.createCommandEncoder();
    {
      const renderPass = t.beginRenderPass(commandEncoder);
      renderPass.setPipeline(pipeline2);
      renderPass.setVertexBuffer(0, vertexBuffer1);
      renderPass.setVertexBuffer(1, vertexBuffer2);
      renderPass.draw(3);
      renderPass.end();
    }
    {
      const renderPass = t.beginRenderPass(commandEncoder);
      renderPass.setPipeline(pipeline1);
      renderPass.draw(3);
      renderPass.end();
    }

    t.expectValidationError(() => {
      commandEncoder.finish();
    });
  }
});
