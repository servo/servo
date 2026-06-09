/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for GPUCanvasContext.getCurrentTexture.
`;import { SkipTestCase } from '../../../common/framework/fixture.js';
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { timeout } from '../../../common/util/timeout.js';
import { assert, unreachable } from '../../../common/util/util.js';
import { AllFeaturesMaxLimitsGPUTest } from '../../gpu_test.js';
import { kAllCanvasTypes, createCanvas } from '../../util/create_elements.js';

const kFormat = 'bgra8unorm';

function expectSingleColorInCanvas(
t,
canvas,
expectedColor,
tolerance = 0.01)
{
  const { width, height } = canvas;
  const copy = new OffscreenCanvas(width, height);
  const copyCtx = copy.getContext('2d');
  copyCtx.drawImage(canvas, 0, 0);
  const imageData = copyCtx.getImageData(0, 0, width, height);
  for (let i = 0; i < imageData.data.length; i += 4) {
    expectedColor.forEach((expected, ch) => {
      const actual = imageData.data[i + ch] / 255;
      const diff = Math.abs(actual - expected);
      t.expect(
        diff < tolerance,
        () =>
        `at ${i % width}x${
        i / width
        }, channel: ${ch}, expected: ${expected}, actual: ${actual}, <  ${tolerance}`
      );
    });
  }
}

class GPUContextTest extends AllFeaturesMaxLimitsGPUTest {
  initCanvasContext(
  canvasType = 'onscreen',
  usage = 0)
  {
    const canvas = createCanvas(this, canvasType, 2, 2);
    if (canvasType === 'onscreen') {
      // To make sure onscreen canvas are visible
      const onscreencanvas = canvas;
      onscreencanvas.style.position = 'fixed';
      onscreencanvas.style.top = '0';
      onscreencanvas.style.left = '0';
      // Set it to transparent so that if multiple canvas are created, they are still visible.
      onscreencanvas.style.opacity = '50%';
      document.body.appendChild(onscreencanvas);
      this.trackForCleanup({
        close() {
          document.body.removeChild(onscreencanvas);
        }
      });
    }
    const ctx = canvas.getContext('webgpu');
    assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

    ctx.configure({
      device: this.device,
      format: kFormat,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC | usage
    });

    return ctx;
  }

  expectTextureDestroyed(texture, expectDestroyed = true) {
    this.expectValidationError(() => {
      // Try using the texture in a render pass. Because it's a canvas texture
      // it should have RENDER_ATTACHMENT usage.
      assert((texture.usage & GPUTextureUsage.RENDER_ATTACHMENT) !== 0);
      const encoder = this.device.createCommandEncoder();
      const pass = encoder.beginRenderPass({
        colorAttachments: [
        {
          view: texture.createView(),
          loadOp: 'clear',
          storeOp: 'store'
        }]

      });
      pass.end();
      // Submitting should generate a validation error if the texture is destroyed.
      this.queue.submit([encoder.finish()]);
    }, expectDestroyed);
  }
}

export const g = makeTestGroup(GPUContextTest);

g.test('configured').
desc(
  `Checks that calling getCurrentTexture requires the context to be configured first, and
  that each call to configure causes getCurrentTexture to return a new texture.`
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes)
).
fn((t) => {
  const canvas = createCanvas(t, t.params.canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  // Calling getCurrentTexture prior to configuration should throw an InvalidStateError exception.
  t.shouldThrow('InvalidStateError', () => {
    ctx.getCurrentTexture();
  });

  // Once the context has been configured getCurrentTexture can be called.
  ctx.configure({
    device: t.device,
    format: kFormat
  });

  let prevTexture = ctx.getCurrentTexture();

  // Calling configure again with different values will change the texture returned.
  ctx.configure({
    device: t.device,
    format: 'bgra8unorm'
  });

  let currentTexture = ctx.getCurrentTexture();
  t.expect(prevTexture !== currentTexture);
  prevTexture = currentTexture;

  // Calling configure again with the same values will still change the texture returned.
  ctx.configure({
    device: t.device,
    format: 'bgra8unorm'
  });

  currentTexture = ctx.getCurrentTexture();
  t.expect(prevTexture !== currentTexture);
  prevTexture = currentTexture;

  // Calling getCurrentTexture after calling unconfigure should throw an InvalidStateError exception.
  ctx.unconfigure();

  t.shouldThrow('InvalidStateError', () => {
    ctx.getCurrentTexture();
  });
});

g.test('single_frames').
desc(`Checks that the value of getCurrentTexture is consistent within a single frame.`).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes)
).
fn((t) => {
  const ctx = t.initCanvasContext(t.params.canvasType);
  const frameTexture = ctx.getCurrentTexture();

  // Calling getCurrentTexture a second time returns the same texture.
  t.expect(frameTexture === ctx.getCurrentTexture());

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: frameTexture.createView(),
      clearValue: [1.0, 0.0, 0.0, 1.0],
      loadOp: 'clear',
      storeOp: 'store'
    }]

  });
  pass.end();
  t.device.queue.submit([encoder.finish()]);

  // Calling getCurrentTexture after performing some work on the texture returns the same texture.
  t.expect(frameTexture === ctx.getCurrentTexture());

  // Ensure that getCurrentTexture does not clear the texture.
  t.expectSingleColor(frameTexture, frameTexture.format, {
    size: [frameTexture.width, frameTexture.height, 1],
    exp: { R: 1, G: 0, B: 0, A: 1 }
  });

  frameTexture.destroy();

  // Calling getCurrentTexture after destroying the texture still returns the same texture.
  t.expect(frameTexture === ctx.getCurrentTexture());
});

g.test('multiple_frames').
desc(`Checks that the value of getCurrentTexture differs across multiple frames.`).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
beginSubcases().
combine('clearTexture', [true, false])
).
beforeAllSubcases((t) => {
  const { canvasType } = t.params;
  t.skipIf(
    canvasType === 'offscreen' && typeof OffscreenCanvas === 'undefined',
    'OffscreenCanvas does not exist in this environment'
  );
  t.skipIf(
    canvasType === 'offscreen' && !('transferToImageBitmap' in OffscreenCanvas.prototype),
    'transferToImageBitmap not supported'
  );
}).
fn((t) => {
  const { canvasType, clearTexture } = t.params;

  return new Promise((resolve) => {
    const ctx = t.initCanvasContext(canvasType);
    let prevTexture;
    let frameCount = 0;

    function frameCheck() {
      const currentTexture = ctx.getCurrentTexture();

      if (prevTexture) {
        // Ensure that each frame a new texture object is returned.
        t.expect(currentTexture !== prevTexture);

        // Ensure that the texture's initial contents are transparent black.
        t.expectSingleColor(currentTexture, currentTexture.format, {
          size: [currentTexture.width, currentTexture.height, 1],
          exp: { R: 0, G: 0, B: 0, A: 0 }
        });
      }

      if (clearTexture) {
        // Fill the texture with a non-zero color, to test that texture
        // contents don't carry over from frame to frame.
        const encoder = t.device.createCommandEncoder();
        const pass = encoder.beginRenderPass({
          colorAttachments: [
          {
            view: currentTexture.createView(),
            clearValue: [1.0, 0.0, 0.0, 1.0],
            loadOp: 'clear',
            storeOp: 'store'
          }]

        });
        pass.end();
        t.device.queue.submit([encoder.finish()]);
      }

      prevTexture = currentTexture;

      if (frameCount++ < 5) {
        // Which method will be used to begin a new "frame"?
        switch (canvasType) {
          case 'onscreen':
            requestAnimationFrame(frameCheck);
            break;
          case 'offscreen':{
              ctx.canvas.transferToImageBitmap();
              frameCheck();
              break;
            }
          default:
            unreachable();
        }
      } else {
        resolve();
      }
    }

    // Render the first frame immediately. The rest will be triggered recursively.
    frameCheck();
  });
});

g.test('resize').
desc(`Checks the value of getCurrentTexture differs when the canvas is resized.`).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes)
).
fn((t) => {
  const ctx = t.initCanvasContext(t.params.canvasType);
  let prevTexture = ctx.getCurrentTexture();

  // Trigger a resize by changing the width.
  ctx.canvas.width = 4;

  t.expectTextureDestroyed(prevTexture);

  // When the canvas resizes the texture returned by getCurrentTexture should immediately begin
  // returning a new texture matching the update dimensions.
  let currentTexture = ctx.getCurrentTexture();
  t.expect(prevTexture !== currentTexture);
  t.expect(currentTexture.width === ctx.canvas.width);
  t.expect(currentTexture.height === ctx.canvas.height);

  // The width and height of the previous texture should remain unchanged.
  t.expect(prevTexture.width === 2);
  t.expect(prevTexture.height === 2);
  prevTexture = currentTexture;

  // Ensure that texture contents are transparent black.
  t.expectSingleColor(currentTexture, currentTexture.format, {
    size: [currentTexture.width, currentTexture.height, 1],
    exp: { R: 0, G: 0, B: 0, A: 0 }
  });

  // Trigger a resize by changing the height.
  ctx.canvas.height = 4;

  // Check to ensure the texture is resized again.
  currentTexture = ctx.getCurrentTexture();
  t.expect(prevTexture !== currentTexture);
  t.expect(currentTexture.width === ctx.canvas.width);
  t.expect(currentTexture.height === ctx.canvas.height);
  t.expect(prevTexture.width === 4);
  t.expect(prevTexture.height === 2);

  // Ensure that texture contents are transparent black.
  t.expectSingleColor(currentTexture, currentTexture.format, {
    size: [currentTexture.width, currentTexture.height, 1],
    exp: { R: 0, G: 0, B: 0, A: 0 }
  });

  // HTMLCanvasElement behaves differently than OffscreenCanvas
  if (t.params.canvasType === 'onscreen') {
    // Ensure canvas goes back to defaults when set to negative numbers.
    ctx.canvas.width = -1;
    currentTexture = ctx.getCurrentTexture();
    t.expect(currentTexture.width === 300);
    t.expect(currentTexture.height === 4);

    ctx.canvas.height = -1;
    currentTexture = ctx.getCurrentTexture();
    t.expect(currentTexture.width === 300);
    t.expect(currentTexture.height === 150);

    // Setting the canvas width and height values to their current values should
    // still trigger a change in the texture.
    prevTexture = ctx.getCurrentTexture();
    const { width, height } = ctx.canvas;
    ctx.canvas.width = width;
    ctx.canvas.height = height;

    t.expectTextureDestroyed(prevTexture);

    currentTexture = ctx.getCurrentTexture();
    t.expect(prevTexture !== currentTexture);
  }
});

g.test('expiry').
desc(
  `
Test automatic WebGPU canvas texture expiry on all canvas types with the following requirements:
- getCurrentTexture returns the same texture object until the next task:
  - after previous frame update the rendering
  - before current frame update the rendering
  - in a microtask off the current frame task
- getCurrentTexture returns a new texture object and the old texture object becomes invalid
  as soon as possible after HTML update the rendering.

TODO: test more canvas types, and ways to update the rendering
- if on a different thread, expiry happens when the worker updates its rendering (worker "rPAF") OR transferToImageBitmap is called
- [draw, transferControlToOffscreen, then canvas is displayed] on either {main thread, or transferred to worker}
- [draw, canvas is displayed, then transferControlToOffscreen] on either {main thread, or transferred to worker}
- reftests for the above 2 (what gets displayed when the canvas is displayed)
- with canvas element added to DOM or not (applies to other canvas tests as well)
  - canvas is added to DOM after being rendered
  - canvas is already in DOM but becomes visible after being rendered
  `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
combine('prevFrameCallsite', ['runInNewCanvasFrame', 'requestAnimationFrame']).
combine('getCurrentTextureAgain', [true, false])
).
beforeAllSubcases((t) => {
  if (
  t.params.prevFrameCallsite === 'requestAnimationFrame' &&
  typeof requestAnimationFrame === 'undefined')
  {
    throw new SkipTestCase('requestAnimationFrame not available');
  }
}).
fn((t) => {
  const { canvasType, prevFrameCallsite, getCurrentTextureAgain } = t.params;
  const ctx = t.initCanvasContext(t.params.canvasType);
  // Create a bindGroupLayout to test invalid texture view usage later.
  const bgl = t.device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      texture: {}
    }]

  });

  // The fn is called immediately after previous frame updating the rendering.
  // Polyfill by calling the callback by setTimeout, in the requestAnimationFrame callback (for onscreen canvas)
  // or after transferToImageBitmap (for offscreen canvas).
  function runInNewCanvasFrame(fn) {
    switch (canvasType) {
      case 'onscreen':
        requestAnimationFrame(() => timeout(fn));
        break;
      case 'offscreen':
        // for offscreen canvas, after calling transferToImageBitmap, we are in a new frame immediately
        ctx.canvas.transferToImageBitmap();
        fn();
        break;
      default:
        unreachable();
    }
  }

  function checkGetCurrentTexture() {
    // Call getCurrentTexture on previous frame.
    const prevTexture = ctx.getCurrentTexture();

    // Call getCurrentTexture immediately after the frame, the texture object should stay the same.
    queueMicrotask(() => {
      if (getCurrentTextureAgain) {
        t.expect(prevTexture === ctx.getCurrentTexture());
      }

      // Call getCurrentTexture in a new frame.
      // It should expire the previous texture object return a new texture object by the next frame by then.
      // Call runInNewCanvasFrame in the micro task to make sure the new frame run after the getCurrentTexture in the micro task for offscreen canvas.
      runInNewCanvasFrame(() => {
        if (getCurrentTextureAgain) {
          t.expect(prevTexture !== ctx.getCurrentTexture());
        }

        // Event when prevTexture expired, createView should still succeed anyway.
        const prevTextureView = prevTexture.createView();
        // Using the invalid view should fail if it expires.
        t.expectValidationError(() => {
          t.device.createBindGroup({
            layout: bgl,
            entries: [{ binding: 0, resource: prevTextureView }]
          });
        });
      });
    });
  }

  switch (prevFrameCallsite) {
    case 'runInNewCanvasFrame':
      runInNewCanvasFrame(checkGetCurrentTexture);
      break;
    case 'requestAnimationFrame':
      requestAnimationFrame(checkGetCurrentTexture);
      break;
    default:
      break;
  }
});

g.test('compatibility').
desc(
  `
Test that the texture returned from getCurrentTexture has textureBindingViewDimension
and that it's undefined in core and '2d' in compatibility mode.
  `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes)
).
fn((t) => {
  const { canvasType } = t.params;
  const ctx = t.initCanvasContext(canvasType);
  const texture = ctx.getCurrentTexture();
  t.expect(() => 'textureBindingViewDimension' in texture);

  const expected = t.isCompatibility ? '2d' : undefined;
  t.expect(texture.textureBindingViewDimension === expected);
});

g.test('usage_as_color_attachment_and_resolve_target').
desc(
  `
Test that the texture returned from getCurrentTexture can be used as a color attachment.
  `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
combine('withView', [true, false]).
combine('asResolveTarget', [true, false])
).
fn((t) => {
  const { canvasType, withView, asResolveTarget } = t.params;
  const ctx = t.initCanvasContext(canvasType);
  const texture = ctx.getCurrentTexture();

  const clearValue = [1, 0.5, 0, 1];
  const encoder = t.device.createCommandEncoder();

  if (asResolveTarget) {
    const srcTexture = t.createTextureTracked({
      size: [texture.width, texture.height, 1],
      format: texture.format,
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      sampleCount: 4
    });
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: withView ? srcTexture.createView() : srcTexture,
        resolveTarget: withView ? texture.createView() : texture,
        loadOp: 'clear',
        storeOp: 'store',
        clearValue
      }]

    });
    pass.end();
  } else {
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: withView ? texture.createView() : texture,
        loadOp: 'clear',
        storeOp: 'store',
        clearValue
      }]

    });
    pass.end();
  }
  t.device.queue.submit([encoder.finish()]);
  expectSingleColorInCanvas(t, ctx.canvas, clearValue);
});

g.test('usage_as_texture_binding').
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
combine('withView', [true, false])
).
fn((t) => {
  const { canvasType, withView } = t.params;
  const ctx = t.initCanvasContext(canvasType, GPUTextureUsage.TEXTURE_BINDING);
  const texture = ctx.getCurrentTexture();

  const clearValue = [1, 1, 0, 1];
  const encoder = t.device.createCommandEncoder();

  // clear canvas to solid color.
  {
    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: texture.createView(),
        loadOp: 'clear',
        storeOp: 'store',
        clearValue
      }]

    });
    pass.end();
  }

  // Render canvas to texture
  {
    const module = t.device.createShaderModule({
      code: `
          @vertex
          fn vs(@builtin(vertex_index) index : u32) -> @builtin(position) vec4f {
            let pos = array(
              vec2f(-1, -1),
              vec2f(-1,  3),
              vec2f( 3, -1),
            );
            return vec4f(pos[index], 0, 1);
          }

          @group(0) @binding(0) var tex: texture_2d<f32>;

          @fragment
          fn fs(@builtin(position) pos: vec4f) -> @location(0) vec4f {
            return textureLoad(tex, vec2u(pos.xy), 0);
          }
        `
    });

    const pipeline = t.device.createRenderPipeline({
      layout: 'auto',
      vertex: { module },
      fragment: { module, targets: [{ format: texture.format }] }
    });

    const bindGroup = t.device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
      {
        binding: 0,
        resource: withView ? texture.createView() : texture
      }]

    });

    const dstTexture = t.createTextureTracked({
      size: [texture.width, texture.height, 1],
      format: texture.format,
      usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.COPY_SRC
    });

    const pass = encoder.beginRenderPass({
      colorAttachments: [
      {
        view: dstTexture.createView(),
        loadOp: 'clear',
        storeOp: 'store'
      }]

    });
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.draw(3);
    pass.end();
    t.device.queue.submit([encoder.finish()]);

    t.expectSingleColor(dstTexture, dstTexture.format, {
      size: [dstTexture.width, dstTexture.height, 1],
      exp: {
        R: clearValue[0],
        G: clearValue[1],
        B: clearValue[2],
        A: clearValue[3]
      }
    });
  }
});