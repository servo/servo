/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests for GPUCanvasContext.configure.

TODO:
- Test colorSpace
- Test toneMapping
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { assert } from '../../../common/util/util.js';
import { kCanvasTextureFormats, kTextureUsages } from '../../capability_info.js';
import { GPUConst } from '../../constants.js';
import {
  kAllTextureFormats,
  kFeaturesForFormats,
  filterFormatsByFeature,
  viewCompatible } from
'../../format_info.js';
import { GPUTest } from '../../gpu_test.js';
import { kAllCanvasTypes, createCanvas } from '../../util/create_elements.js';

export const g = makeTestGroup(GPUTest);

g.test('defaults').
desc(
  `
    Ensure that the defaults for GPUCanvasConfiguration are correct.
    `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes)
).
fn((t) => {
  const { canvasType } = t.params;
  const canvas = createCanvas(t, canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  ctx.configure({
    device: t.device,
    format: 'rgba8unorm'
  });

  const configuration = ctx.getConfiguration();
  assert(configuration !== null);
  t.expect(configuration.device === t.device);
  t.expect(configuration.format === 'rgba8unorm');
  t.expect(configuration.usage === GPUTextureUsage.RENDER_ATTACHMENT);
  t.expect(configuration.viewFormats.length === 0);
  t.expect(configuration.colorSpace === 'srgb');
  t.expect(configuration.toneMapping.mode === 'standard');
  t.expect(configuration.alphaMode === 'opaque');

  const currentTexture = ctx.getCurrentTexture();
  t.expect(currentTexture.format === 'rgba8unorm');
  t.expect(currentTexture.usage === GPUTextureUsage.RENDER_ATTACHMENT);
  t.expect(currentTexture.dimension === '2d');
  t.expect(currentTexture.width === canvas.width);
  t.expect(currentTexture.height === canvas.height);
  t.expect(currentTexture.depthOrArrayLayers === 1);
  t.expect(currentTexture.mipLevelCount === 1);
  t.expect(currentTexture.sampleCount === 1);
});

g.test('device').
desc(
  `
    Ensure that configure reacts appropriately to various device states.
    `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes)
).
fn((t) => {
  const { canvasType } = t.params;
  const canvas = createCanvas(t, canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  // getConfiguration returns null before configure.
  t.expect(ctx.getConfiguration() === null);

  // Calling configure without a device should throw a TypeError.
  t.shouldThrow('TypeError', () => {
    ctx.configure({
      format: 'rgba8unorm'
    });
  });

  // Device is not configured, so getCurrentTexture will throw an InvalidStateError.
  t.shouldThrow('InvalidStateError', () => {
    ctx.getCurrentTexture();
  });

  // Calling configure with a device should succeed.
  ctx.configure({
    device: t.device,
    format: 'rgba8unorm',
    alphaMode: 'opaque'
  });

  // getConfiguration will succeed after configure.
  const configuration = ctx.getConfiguration();
  assert(configuration !== null);
  t.expect(configuration.device === t.device);
  t.expect(configuration.format === 'rgba8unorm');
  t.expect(configuration.usage === GPUTextureUsage.RENDER_ATTACHMENT);
  t.expect(configuration.viewFormats.length === 0);
  t.expect(configuration.colorSpace === 'srgb');
  t.expect(configuration.toneMapping.mode === 'standard');
  t.expect(configuration.alphaMode === 'opaque');

  // getCurrentTexture will succeed with a valid device.
  ctx.getCurrentTexture();

  // Unconfiguring should cause the device to be cleared.
  ctx.unconfigure();
  t.shouldThrow('InvalidStateError', () => {
    ctx.getCurrentTexture();
  });

  // getConfiguration returns null after unconfigure.
  t.expect(ctx.getConfiguration() === null);

  // Should be able to successfully configure again after unconfiguring.
  ctx.configure({
    device: t.device,
    format: 'rgba8unorm',
    alphaMode: 'premultiplied'
  });
  ctx.getCurrentTexture();

  // getConfiguration will succeed after configure.
  const newConfiguration = ctx.getConfiguration();
  assert(newConfiguration !== null);
  t.expect(newConfiguration.device === t.device);
  t.expect(newConfiguration.format === 'rgba8unorm');
  t.expect(newConfiguration.usage === GPUTextureUsage.RENDER_ATTACHMENT);
  t.expect(newConfiguration.viewFormats.length === 0);
  t.expect(newConfiguration.colorSpace === 'srgb');
  t.expect(newConfiguration.toneMapping.mode === 'standard');
  t.expect(newConfiguration.alphaMode === 'premultiplied');
});

g.test('format').
desc(
  `
    Ensure that only valid texture formats are allowed when calling configure.
    `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
combine('format', kAllTextureFormats)
).
beforeAllSubcases((t) => {
  t.selectDeviceForTextureFormatOrSkipTestCase(t.params.format);
}).
fn((t) => {
  const { canvasType, format } = t.params;
  const canvas = createCanvas(t, canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  // Would prefer to use kCanvasTextureFormats.includes(format), but that's giving TS errors.
  let validFormat = false;
  for (const canvasFormat of kCanvasTextureFormats) {
    if (format === canvasFormat) {
      validFormat = true;
      break;
    }
  }

  if (validFormat) {
    ctx.configure({
      device: t.device,
      format
    });
    const configuration = ctx.getConfiguration();
    t.expect(configuration.format === format);
  } else {
    t.shouldThrow('TypeError', () => {
      ctx.configure({
        device: t.device,
        format
      });
    });
  }
});

g.test('usage').
desc(
  `
    Ensure that getCurrentTexture returns a texture with the configured usages.
    `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
beginSubcases().
expand('usage', () => {
  const usageSet = new Set();
  for (const usage0 of kTextureUsages) {
    for (const usage1 of kTextureUsages) {
      usageSet.add(usage0 | usage1);
    }
  }
  return usageSet;
})
).
fn((t) => {
  const { canvasType, usage } = t.params;
  const canvas = createCanvas(t, canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  ctx.configure({
    device: t.device,
    format: 'rgba8unorm',
    usage
  });

  const configuration = ctx.getConfiguration();
  t.expect(configuration.usage === usage);

  const currentTexture = ctx.getCurrentTexture();
  t.expect(currentTexture instanceof GPUTexture);
  t.expect(currentTexture.usage === usage);

  // Try to use the texture with the given usage

  if (usage & GPUConst.TextureUsage.RENDER_ATTACHMENT) {
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

  if (usage & GPUConst.TextureUsage.TEXTURE_BINDING) {
    const bgl = t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        texture: {}
      }]

    });

    t.device.createBindGroup({
      layout: bgl,
      entries: [
      {
        binding: 0,
        resource: currentTexture.createView()
      }]

    });
  }

  if (usage & GPUConst.TextureUsage.STORAGE_BINDING) {
    const bgl = t.device.createBindGroupLayout({
      entries: [
      {
        binding: 0,
        visibility: GPUShaderStage.FRAGMENT,
        storageTexture: { access: 'write-only', format: currentTexture.format }
      }]

    });

    t.device.createBindGroup({
      layout: bgl,
      entries: [
      {
        binding: 0,
        resource: currentTexture.createView()
      }]

    });
  }

  if (usage & GPUConst.TextureUsage.COPY_DST) {
    const rgbaData = new Uint8Array([255, 0, 0, 255]);

    t.device.queue.writeTexture({ texture: currentTexture }, rgbaData, {}, [1, 1, 1]);
  }

  if (usage & GPUConst.TextureUsage.COPY_SRC) {
    const size = [currentTexture.width, currentTexture.height, 1];
    const dstTexture = t.createTextureTracked({
      format: currentTexture.format,
      usage: GPUTextureUsage.COPY_DST,
      size
    });

    const encoder = t.device.createCommandEncoder();
    encoder.copyTextureToTexture({ texture: currentTexture }, { texture: dstTexture }, size);
    t.device.queue.submit([encoder.finish()]);
  }
});

g.test('alpha_mode').
desc(
  `
    Ensure that all valid alphaMode values are allowed when calling configure.
    `
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
beginSubcases().
combine('alphaMode', ['opaque', 'premultiplied'])
).
fn((t) => {
  const { canvasType, alphaMode } = t.params;
  const canvas = createCanvas(t, canvasType, 2, 2);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  ctx.configure({
    device: t.device,
    format: 'rgba8unorm',
    alphaMode
  });

  const configuration = ctx.getConfiguration();
  t.expect(configuration.alphaMode === alphaMode);

  const currentTexture = ctx.getCurrentTexture();
  t.expect(currentTexture instanceof GPUTexture);
});

g.test('size_zero_before_configure').
desc(`Ensure a validation error is raised in configure() if the size of the canvas is zero.`).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
combine('zeroDimension', ['width', 'height'])
).
fn((t) => {
  const { canvasType, zeroDimension } = t.params;
  const canvas = createCanvas(t, canvasType, 1, 1);
  canvas[zeroDimension] = 0;
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  // Validation error, the canvas size is 0 which doesn't make a valid GPUTextureDescriptor.
  t.expectValidationError(() => {
    ctx.configure({
      device: t.device,
      format: 'bgra8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    });
  });

  canvas[zeroDimension] = 1;

  // The size being incorrect doesn't make for an invalid configuration. Now that it is fixed
  // getting textures from the canvas should work.
  const currentTexture = ctx.getCurrentTexture();

  // Try rendering to it even!
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
});

g.test('size_zero_after_configure').
desc(
  `Ensure a validation error is raised after configure() if the size of the canvas becomes zero.`
).
params((u) =>
u //
.combine('canvasType', kAllCanvasTypes).
combine('zeroDimension', ['width', 'height'])
).
fn((t) => {
  const { canvasType, zeroDimension } = t.params;
  const canvas = createCanvas(t, canvasType, 1, 1);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  ctx.configure({
    device: t.device,
    format: 'bgra8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT
  });

  canvas[zeroDimension] = 0;

  // The size is incorrect, we should be getting an error texture and a validation error.
  let currentTexture;
  t.expectValidationError(() => {
    currentTexture = ctx.getCurrentTexture();
  });

  t.expect(currentTexture[zeroDimension] === 0);

  // Using the texture should produce a validation error.
  t.expectValidationError(() => {
    currentTexture.createView();
  });
});

g.test('viewFormats').
desc(
  `Test the validation that viewFormats are compatible with the format (for all canvas format / view formats)`
).
params((u) =>
u.
combine('canvasType', kAllCanvasTypes).
combine('format', kCanvasTextureFormats).
combine('viewFormatFeature', kFeaturesForFormats).
beginSubcases().
expand('viewFormat', ({ viewFormatFeature }) =>
filterFormatsByFeature(viewFormatFeature, kAllTextureFormats)
)
).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase([t.params.viewFormatFeature]);
}).
fn((t) => {
  const { canvasType, format, viewFormat } = t.params;

  t.skipIfTextureFormatNotSupported(viewFormat);

  const canvas = createCanvas(t, canvasType, 1, 1);
  const ctx = canvas.getContext('webgpu');
  assert(ctx instanceof GPUCanvasContext, 'Failed to get WebGPU context from canvas');

  const compatible = viewCompatible(t.isCompatibility, format, viewFormat);

  // Test configure() produces an error if the formats aren't compatible.
  t.expectValidationError(() => {
    ctx.configure({
      device: t.device,
      format,
      viewFormats: [viewFormat]
    });
  }, !compatible);

  const viewFormats = ctx.getConfiguration().viewFormats;
  t.expect(viewFormats[0] === viewFormat);

  // Likewise for getCurrentTexture().
  let currentTexture;
  t.expectValidationError(() => {
    currentTexture = ctx.getCurrentTexture();
  }, !compatible);

  // The returned texture is an error texture.
  t.expectValidationError(() => {
    currentTexture.createView();
  }, !compatible);
});