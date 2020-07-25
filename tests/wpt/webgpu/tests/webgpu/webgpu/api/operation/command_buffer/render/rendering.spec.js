/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = '';
import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('fullscreen_quad').fn(async t => {
  const dst = t.device.createBuffer({
    size: 4,
    usage: GPUBufferUsage.COPY_SRC | GPUBufferUsage.COPY_DST,
  });

  const colorAttachment = t.device.createTexture({
    format: 'rgba8unorm',
    size: { width: 1, height: 1, depth: 1 },
    usage: GPUTextureUsage.COPY_SRC | GPUTextureUsage.OUTPUT_ATTACHMENT,
  });

  const colorAttachmentView = colorAttachment.createView();

  const vertexModule = t.makeShaderModule('vertex', {
    glsl: `
      #version 310 es
      void main() {
        const vec2 pos[3] = vec2[3](
            vec2(-1.f, -3.f), vec2(3.f, 1.f), vec2(-1.f, 1.f));
        gl_Position = vec4(pos[gl_VertexIndex], 0.f, 1.f);
      }
    `,
  });

  const fragmentModule = t.makeShaderModule('fragment', {
    glsl: `
      #version 310 es
      precision mediump float;
      layout(location = 0) out vec4 fragColor;
      void main() {
        fragColor = vec4(0.0, 1.0, 0.0, 1.0);
      }
    `,
  });

  const pl = t.device.createPipelineLayout({ bindGroupLayouts: [] });
  const pipeline = t.device.createRenderPipeline({
    vertexStage: { module: vertexModule, entryPoint: 'main' },
    fragmentStage: { module: fragmentModule, entryPoint: 'main' },
    layout: pl,
    primitiveTopology: 'triangle-list',
    rasterizationState: {
      frontFace: 'ccw',
    },

    colorStates: [{ format: 'rgba8unorm', alphaBlend: {}, colorBlend: {} }],
    vertexState: {
      indexFormat: 'uint16',
      vertexBuffers: [],
    },
  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
      {
        attachment: colorAttachmentView,
        storeOp: 'store',
        loadValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      },
    ],
  });

  pass.setPipeline(pipeline);
  pass.draw(3, 1, 0, 0);
  pass.endPass();
  encoder.copyTextureToBuffer(
    { texture: colorAttachment, mipLevel: 0, origin: { x: 0, y: 0, z: 0 } },
    { buffer: dst, bytesPerRow: 256 },
    { width: 1, height: 1, depth: 1 }
  );

  t.device.defaultQueue.submit([encoder.finish()]);

  t.expectContents(dst, new Uint8Array([0x00, 0xff, 0x00, 0xff]));
});
