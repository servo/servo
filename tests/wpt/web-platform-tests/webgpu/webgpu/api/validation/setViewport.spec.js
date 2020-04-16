/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
setViewport validation tests.
`;
import { TestGroup } from '../../../common/framework/test_group.js';
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
g.test('use of setViewport', async t => {
  const {
    x,
    y,
    width,
    height,
    minDepth,
    maxDepth,
    _success
  } = t.params;
  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = t.beginRenderPass(commandEncoder);
  renderPass.setViewport(x, y, width, height, minDepth, maxDepth);
  renderPass.endPass();
  t.expectValidationError(() => {
    commandEncoder.finish();
  }, !_success);
}).params([{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 0,
  maxDepth: 1,
  _success: true
}, // Basic use
{
  x: 0,
  y: 0,
  width: 0,
  height: 1,
  minDepth: 0,
  maxDepth: 1,
  _success: false
}, // Width of zero is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 0,
  minDepth: 0,
  maxDepth: 1,
  _success: false
}, // Height of zero is not allowed
{
  x: 0,
  y: 0,
  width: 0,
  height: 0,
  minDepth: 0,
  maxDepth: 1,
  _success: false
}, // Both width and height of zero are not allowed
{
  x: -1,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 0,
  maxDepth: 1,
  _success: true
}, // Negative x is allowed
{
  x: 0,
  y: -1,
  width: 1,
  height: 1,
  minDepth: 0,
  maxDepth: 1,
  _success: true
}, // Negative y is allowed
{
  x: 0,
  y: 0,
  width: -1,
  height: 1,
  minDepth: 0,
  maxDepth: 1,
  _success: false
}, // Negative width is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: -1,
  minDepth: 0,
  maxDepth: 1,
  _success: false
}, // Negative height is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: -1,
  maxDepth: 1,
  _success: false
}, // Negative minDepth is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 0,
  maxDepth: -1,
  _success: false
}, // Negative maxDepth is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 10,
  maxDepth: 1,
  _success: false
}, // minDepth greater than 1 is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 0,
  maxDepth: 10,
  _success: false
}, // maxDepth greater than 1 is not allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 0.5,
  maxDepth: 0.5,
  _success: true
}, // minDepth equal to maxDepth is allowed
{
  x: 0,
  y: 0,
  width: 1,
  height: 1,
  minDepth: 0.8,
  maxDepth: 0.5,
  _success: true
}, // minDepth greater than maxDepth is allowed
{
  x: 0,
  y: 0,
  width: TEXTURE_WIDTH + 1,
  height: TEXTURE_HEIGHT + 1,
  minDepth: 0,
  maxDepth: 1,
  _success: true
} // Viewport larger than the framebuffer is allowed
]);
//# sourceMappingURL=setViewport.spec.js.map