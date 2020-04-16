/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
setBlendColor validation tests.
`;
import { TestGroup } from '../../../common/framework/test_group.js';
import { ValidationTest } from './validation_test.js'; // TODO: Move beginRenderPass to a Fixture class.

class F extends ValidationTest {
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
g.test('basic use of setBlendColor', t => {
  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = t.beginRenderPass(commandEncoder);
  renderPass.setBlendColor({
    r: 0,
    g: 0,
    b: 0,
    a: 0
  });
  renderPass.endPass();
  commandEncoder.finish();
});
g.test('setBlendColor allows any number value', t => {
  const values = [Number.MIN_SAFE_INTEGER, Number.MAX_SAFE_INTEGER];

  for (const value of values) {
    const commandEncoder = t.device.createCommandEncoder();
    const renderPass = t.beginRenderPass(commandEncoder);
    renderPass.setBlendColor({
      r: value,
      g: value,
      b: value,
      a: value
    });
    renderPass.endPass();
    commandEncoder.finish();
  }
});
//# sourceMappingURL=setBlendColor.spec.js.map