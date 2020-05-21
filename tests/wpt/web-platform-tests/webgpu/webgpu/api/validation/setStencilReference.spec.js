/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
setStencilReference validation tests.
`;
import { poptions } from '../../../common/framework/params_builder.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { ValidationTest } from './validation_test.js'; // TODO: Move this fixture class to a common file.

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

export const g = makeTestGroup(F);
g.test('use_of_setStencilReference').params(poptions('reference', [0, 0xffffffff])).fn(t => {
  const {
    reference
  } = t.params;
  const commandEncoder = t.device.createCommandEncoder();
  const renderPass = t.beginRenderPass(commandEncoder);
  renderPass.setStencilReference(reference);
  renderPass.endPass();
  commandEncoder.finish();
});
//# sourceMappingURL=setStencilReference.spec.js.map