/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { runRefTest } from './gpu_ref_test.js';runRefTest((t) => {
  function draw(canvasId, format) {
    const canvas = document.getElementById(canvasId);

    const ctx = canvas.getContext('webgpu');
    ctx.configure({
      device: t.device,
      format
    });

    const colorAttachment = ctx.getCurrentTexture();
    const colorAttachmentView = colorAttachment.createView();

    const encoder = t.device.createCommandEncoder();
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: colorAttachmentView,
        clearValue: { r: 0.4, g: 1.0, b: 0.0, a: 1.0 },
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.end();
    t.device.queue.submit([encoder.finish()]);
  }

  draw('cvs0', 'bgra8unorm');
  draw('cvs1', 'rgba8unorm');
  draw('cvs2', 'rgba16float');
});