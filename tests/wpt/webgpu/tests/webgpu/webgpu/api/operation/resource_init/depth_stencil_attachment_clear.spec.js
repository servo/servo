/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description =
  'Test uninitialized textures are initialized to zero when used as a depth/stencil attachment.';
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { unreachable } from '../../../../common/framework/util/util.js';

import {
  ReadMethod,
  TextureZeroInitTest,
  initializedStateAsDepth,
  initializedStateAsStencil,
} from './texture_zero_init_test.js';

class DepthStencilAttachmentClearTest extends TextureZeroInitTest {
  // Construct a pipeline which will render a single triangle with depth
  // equal to |initializeStateAsDepth(state)|. The depth compare function
  // is set to "equal" so the fragment shader will only write 1.0 to the
  // R8Unorm output if the depth buffer contains exactly the expected value.
  getDepthTestReadbackPipeline(state, format, sampleCount) {
    return this.device.createRenderPipeline({
      vertexStage: {
        entryPoint: 'main',
        module: this.makeShaderModule('vertex', {
          glsl: `#version 310 es
          void main() {
            const vec2 pos[3] = vec2[3](
                vec2(-1.f, -3.f), vec2(3.f, 1.f), vec2(-1.f, 1.f));
            gl_Position = vec4(pos[gl_VertexIndex], 0.f, 1.f);
          }
          `,
        }),
      },

      fragmentStage: {
        entryPoint: 'main',
        module: this.makeShaderModule('fragment', {
          glsl: `#version 310 es
          precision highp float;
          layout(location = 0) out float outSuccess;

          void main() {
            gl_FragDepth = float(${initializedStateAsDepth(state)});
            outSuccess = 1.0;
          }
          `,
        }),
      },

      colorStates: [
        {
          format: 'r8unorm',
        },
      ],

      depthStencilState: {
        format,
        depthCompare: 'equal',
      },

      primitiveTopology: 'triangle-list',
      sampleCount,
    });
  }

  // Construct a pipeline which will render a single triangle.
  // The stencil compare function is set to "equal" so the fragment shader
  // will only write 1.0 to the R8Unorm output if the stencil buffer contains
  // exactly the stencil reference value.
  getStencilTestReadbackPipeline(format, sampleCount) {
    return this.device.createRenderPipeline({
      vertexStage: {
        entryPoint: 'main',
        module: this.makeShaderModule('vertex', {
          glsl: `#version 310 es
          void main() {
            const vec2 pos[3] = vec2[3](
                vec2(-1.f, -3.f), vec2(3.f, 1.f), vec2(-1.f, 1.f));
            gl_Position = vec4(pos[gl_VertexIndex], 0.f, 1.f);
          }
          `,
        }),
      },

      fragmentStage: {
        entryPoint: 'main',
        module: this.makeShaderModule('fragment', {
          glsl: `#version 310 es
          precision highp float;
          layout(location = 0) out float outSuccess;

          void main() {
            outSuccess = 1.0;
          }
          `,
        }),
      },

      colorStates: [
        {
          format: 'r8unorm',
        },
      ],

      depthStencilState: {
        format,
        stencilFront: {
          compare: 'equal',
        },

        stencilBack: {
          compare: 'equal',
        },
      },

      primitiveTopology: 'triangle-list',
      sampleCount,
    });
  }

  // Check the contents by running either a depth or stencil test. The test will
  // render 1.0 to an R8Unorm texture if the depth/stencil buffer is equal to the expected
  // value. This is done by using a depth compare function and explicitly setting the depth
  // value with gl_FragDepth in the shader, or by using a stencil compare function and
  // setting the stencil reference value in the render pass.
  checkContents(texture, state, subresourceRange) {
    for (const viewDescriptor of this.generateTextureViewDescriptorsForRendering(
      this.params.aspect,
      subresourceRange
    )) {
      const width = this.textureWidth >> viewDescriptor.baseMipLevel;
      const height = this.textureHeight >> viewDescriptor.baseMipLevel;

      const renderTexture = this.device.createTexture({
        size: [width, height, 1],
        format: 'r8unorm',
        usage: GPUTextureUsage.OUTPUT_ATTACHMENT | GPUTextureUsage.COPY_SRC,
        sampleCount: this.params.sampleCount,
      });

      let resolveTexture = undefined;
      let resolveTarget = undefined;
      if (this.params.sampleCount > 1) {
        resolveTexture = this.device.createTexture({
          size: [width, height, 1],
          format: 'r8unorm',
          usage: GPUTextureUsage.OUTPUT_ATTACHMENT | GPUTextureUsage.COPY_SRC,
        });

        resolveTarget = resolveTexture.createView();
      }

      const commandEncoder = this.device.createCommandEncoder();
      const pass = commandEncoder.beginRenderPass({
        colorAttachments: [
          {
            attachment: renderTexture.createView(),
            resolveTarget,
            loadValue: [0, 0, 0, 0],
            storeOp: 'store',
          },
        ],

        depthStencilAttachment: {
          attachment: texture.createView(viewDescriptor),
          depthStoreOp: 'store',
          depthLoadValue: 'load',
          stencilStoreOp: 'store',
          stencilLoadValue: 'load',
        },
      });

      switch (this.params.readMethod) {
        case ReadMethod.DepthTest:
          pass.setPipeline(
            this.getDepthTestReadbackPipeline(state, this.params.format, this.params.sampleCount)
          );

          break;

        case ReadMethod.StencilTest:
          pass.setPipeline(
            this.getStencilTestReadbackPipeline(this.params.format, this.params.sampleCount)
          );

          // Set the stencil reference. The rendering pipeline uses stencil compare function "equal"
          // so this pass will write 1.0 to the output only if the stencil buffer is equal to this
          // reference value.
          pass.setStencilReference(initializedStateAsStencil(state));
          break;

        default:
          unreachable();
      }

      pass.draw(3, 1, 0, 0);
      pass.endPass();

      this.queue.submit([commandEncoder.finish()]);

      this.expectSingleColor(resolveTexture || renderTexture, 'r8unorm', {
        size: [width, height, 1],
        exp: { R: 1 },
      });
    }
  }
}

export const g = makeTestGroup(DepthStencilAttachmentClearTest);

g.test('uninitialized_texture_is_zero')
  .params(TextureZeroInitTest.generateParams([ReadMethod.DepthTest, ReadMethod.StencilTest]))
  .fn(t => t.run());
