/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
Basic command buffer rendering tests.
`;
import { TestGroup } from '../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../gpu_test.js';
export const g = new TestGroup(GPUTest);
g.test('clear', async t => {
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
  const colorAttachmentView = colorAttachment.createView();
  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [{
      attachment: colorAttachmentView,
      loadValue: {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0
      },
      storeOp: 'store'
    }]
  });
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
    bytesPerRow: 256
  }, {
    width: 1,
    height: 1,
    depth: 1
  });
  t.device.defaultQueue.submit([encoder.finish()]);
  t.expectContents(dst, new Uint8Array([0x00, 0xff, 0x00, 0xff]));
});
//# sourceMappingURL=basic.spec.js.map