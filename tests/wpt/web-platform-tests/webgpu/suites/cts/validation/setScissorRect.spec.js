/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
setScissorRect validation tests.
`;
import { TestGroup } from '../../../framework/index.js';
import { ValidationTest } from './validation_test.js';
const TEXTURE_WIDTH = 16;
const TEXTURE_HEIGHT = 16; // TODO: Move this fixture class to a common file.

class F extends ValidationTest {
  beginRenderPass(commandEncoder) {
    const attachmentTexture = this.device.createTexture({
      format: 'rgba8unorm',
      size: {
        width: TEXTURE_WIDTH,
        height: TEXTURE_HEIGHT,
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
g.test('use of setScissorRect', async t => {
  const {
    x,
    y,
    width,
    height,
    success
  } = t.params;
  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = t.beginRenderPass(commandEncoder);
  renderPass.setScissorRect(x, y, width, height);
  renderPass.endPass();
  await t.expectValidationError(() => {
    commandEncoder.finish();
  }, !success);
}).params([{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  success: true
}, // Basic use
{
  x: 0,
  y: 0,
  width: 0,
  height: 1,
  success: false
}, // Width of zero is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 0,
  success: false
}, // Height of zero is not allowed
{
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  success: false
}, // Both width and height of zero are not allowed
{
  x: 0,
  y: 0,
  width: TEXTURE_WIDTH + 1,
  height: TEXTURE_HEIGHT + 1,
  success: true
} // Scissor larger than the framebuffer is allowed
]);
//# sourceMappingURL=setScissorRect.spec.js.map