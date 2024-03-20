/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests using a destroyed texture on a queue.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { unreachable } from '../../../../../common/util/util.js';
import { ValidationTest } from '../../validation_test.js';

export const g = makeTestGroup(ValidationTest);

g.test('writeTexture').
desc(
  `
Tests that using a destroyed texture in writeTexture fails.
- x= {destroyed, not destroyed (control case)}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const texture = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_DST
    })
  );

  if (destroyed) {
    texture.destroy();
  }

  t.expectValidationError(
    () => t.queue.writeTexture({ texture }, new Uint8Array(4), { bytesPerRow: 4 }, [1, 1, 1]),
    destroyed
  );
});

g.test('copyTextureToTexture').
desc(
  `
Tests that using a destroyed texture in copyTextureToTexture fails.
- x= {not destroyed (control case), src destroyed, dst destroyed}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', ['none', 'src', 'dst', 'both'])).
fn((t) => {
  const src = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC
    })
  );
  const dst = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_DST
    })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToTexture({ texture: src }, { texture: dst }, [1, 1, 1]);
  const commandBuffer = encoder.finish();

  let shouldError = true;
  switch (t.params.destroyed) {
    case 'none':
      shouldError = false;
      break;
    case 'src':
      src.destroy();
      break;
    case 'dst':
      dst.destroy();
      break;
    case 'both':
      src.destroy();
      dst.destroy();
      break;
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, shouldError);
});

g.test('copyBufferToTexture').
desc(
  `
Tests that using a destroyed texture in copyBufferToTexture fails.
- x= {not destroyed (control case), dst destroyed}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const buffer = t.trackForCleanup(
    t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_SRC })
  );
  const texture = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_DST
    })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyBufferToTexture({ buffer }, { texture }, [1, 1, 1]);
  const commandBuffer = encoder.finish();

  if (destroyed) {
    texture.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('copyTextureToBuffer').
desc(
  `
Tests that using a destroyed texture in copyTextureToBuffer fails.
- x= {not destroyed (control case), src destroyed}
  `
).
paramsSubcasesOnly((u) => u.combine('destroyed', [false, true])).
fn((t) => {
  const { destroyed } = t.params;
  const texture = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.COPY_SRC
    })
  );
  const buffer = t.trackForCleanup(
    t.device.createBuffer({ size: 4, usage: GPUBufferUsage.COPY_DST })
  );

  const encoder = t.device.createCommandEncoder();
  encoder.copyTextureToBuffer({ texture }, { buffer }, [1, 1, 1]);
  const commandBuffer = encoder.finish();

  if (destroyed) {
    texture.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('setBindGroup').
desc(
  `
Tests that using a destroyed texture referenced by a bindGroup set with setBindGroup fails
- x= {not destroyed (control case), destroyed}
    `
).
paramsSubcasesOnly((u) =>
u.
combine('destroyed', [false, true]).
combine('encoderType', ['compute pass', 'render pass', 'render bundle']).
combine('bindingType', ['texture', 'storageTexture'])
).
fn((t) => {
  const { destroyed, encoderType, bindingType } = t.params;
  const { device } = t;
  const texture = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.STORAGE_BINDING
    })
  );

  const layout = device.createBindGroupLayout({
    entries: [
    {
      binding: 0,
      visibility: GPUShaderStage.COMPUTE,
      [bindingType]: {
        format: texture.format
      }
    }]

  });

  const bindGroup = device.createBindGroup({
    layout,
    entries: [{ binding: 0, resource: texture.createView() }]
  });

  const { encoder, finish } = t.createEncoder(encoderType);
  encoder.setBindGroup(0, bindGroup);
  const commandBuffer = finish();

  if (destroyed) {
    texture.destroy();
  }

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, destroyed);
});

g.test('beginRenderPass').
desc(
  `
Tests that using a destroyed texture referenced by a render pass fails
- x= {not destroyed (control case), colorAttachment destroyed, depthAttachment destroyed, resolveTarget destroyed}
    `
).
paramsSubcasesOnly((u) =>
u.combine('textureToDestroy', [
'none',
'colorAttachment',
'resolveAttachment',
'depthStencilAttachment']
)
).
fn((t) => {
  const { textureToDestroy } = t.params;
  const { device } = t;

  const colorAttachment = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      sampleCount: 4,
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    })
  );

  const resolveAttachment = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    })
  );

  const depthStencilAttachment = t.trackForCleanup(
    t.device.createTexture({
      size: [1, 1, 1],
      format: 'depth32float',
      sampleCount: 4,
      usage: GPUTextureUsage.RENDER_ATTACHMENT
    })
  );

  const encoder = device.createCommandEncoder();
  const pass = encoder.beginRenderPass({
    colorAttachments: [
    {
      view: colorAttachment.createView(),
      resolveTarget: resolveAttachment.createView(),
      loadOp: 'clear',
      storeOp: 'store'
    }],

    depthStencilAttachment: {
      view: depthStencilAttachment.createView(),
      depthClearValue: 0,
      depthLoadOp: 'clear',
      depthStoreOp: 'store'
    }
  });
  pass.end();
  const commandBuffer = encoder.finish();

  switch (textureToDestroy) {
    case 'none':
      break;
    case 'colorAttachment':
      colorAttachment.destroy();
      break;
    case 'resolveAttachment':
      resolveAttachment.destroy();
      break;
    case 'depthStencilAttachment':
      depthStencilAttachment.destroy();
      break;
    default:
      unreachable();
  }

  const shouldError = textureToDestroy !== 'none';

  t.expectValidationError(() => {
    t.queue.submit([commandBuffer]);
  }, shouldError);
});