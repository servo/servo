/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

import { runRefTest } from './gpu_ref_test.js';
runRefTest(async t => {
  const canvas = document.getElementById('gpucanvas');
  const ctx = canvas.getContext('gpupresent');
  const swapChain = ctx.configureSwapChain({
    device: t.device,
    format: 'bgra8unorm'
  });
  const colorAttachment = swapChain.getCurrentTexture();
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
  t.device.defaultQueue.submit([encoder.finish()]);
});
//# sourceMappingURL=canvas_clear.js.map