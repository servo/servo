/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Texture Usages Validation Tests in Same or Different Render Pass Encoders.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { assert, unreachable } from '../../../../../common/util/util.js';
import { ValidationTest } from '../../validation_test.js';






export const kTextureBindingTypes = [
'sampled-texture',
'writeonly-storage-texture',
'readonly-storage-texture',
'readwrite-storage-texture'];

export function IsReadOnlyTextureBindingType(t) {
  return t === 'sampled-texture' || t === 'readonly-storage-texture';
}

class F extends ValidationTest {
  getColorAttachment(
  texture,
  textureViewDescriptor)
  {
    const view = texture.createView(textureViewDescriptor);

    return {
      view,
      clearValue: { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
      loadOp: 'clear',
      storeOp: 'store'
    };
  }

  createBindGroupForTest(
  textureView,
  textureUsage,
  sampleType)
  {
    const bindGroupLayoutEntry = {
      binding: 0,
      visibility: GPUShaderStage.FRAGMENT
    };
    switch (textureUsage) {
      case 'sampled-texture':
        bindGroupLayoutEntry.texture = { viewDimension: '2d-array', sampleType };
        break;
      case 'readonly-storage-texture':
        bindGroupLayoutEntry.storageTexture = {
          access: 'read-only',
          format: 'r32float',
          viewDimension: '2d-array'
        };
        break;
      case 'readwrite-storage-texture':
        bindGroupLayoutEntry.storageTexture = {
          access: 'read-write',
          format: 'r32float',
          viewDimension: '2d-array'
        };
        break;
      case 'writeonly-storage-texture':
        bindGroupLayoutEntry.storageTexture = {
          access: 'write-only',
          format: 'r32float',
          viewDimension: '2d-array'
        };
        break;
      default:
        unreachable();
        break;
    }
    const layout = this.device.createBindGroupLayout({
      entries: [bindGroupLayoutEntry]
    });
    return this.device.createBindGroup({
      layout,
      entries: [{ binding: 0, resource: textureView }]
    });
  }

  isRangeNotOverlapped(start0, end0, start1, end1) {
    assert(start0 <= end0 && start1 <= end1);
    // There are only two possibilities for two non-overlapped ranges:
    // [start0, end0] [start1, end1] or
    // [start1, end1] [start0, end0]
    return end0 < start1 || end1 < start0;
  }
}

export const g = makeTestGroup(F);

const kTextureSize = 16;
const kTextureLevels = 3;
const kTextureLayers = 3;

g.test('subresources,color_attachments').
desc(
  `
  Test that the different subresource of the same texture are allowed to be used as color
  attachments in same / different render pass encoder, while the same subresource is only allowed
  to be used as different color attachments in different render pass encoders.`
).
params((u) =>
u.
combine('layer0', [0, 1]).
combine('level0', [0, 1]).
combine('layer1', [0, 1]).
combine('level1', [0, 1]).
combine('inSamePass', [true, false]).
unless((t) => t.inSamePass && t.level0 !== t.level1)
).
fn((t) => {
  const { layer0, level0, layer1, level1, inSamePass } = t.params;

  const texture = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, kTextureLayers],
    mipLevelCount: kTextureLevels
  });

  const colorAttachment1 = t.getColorAttachment(texture, {
    dimension: '2d',
    baseArrayLayer: layer0,
    arrayLayerCount: 1,
    baseMipLevel: level0,
    mipLevelCount: 1
  });
  const colorAttachment2 = t.getColorAttachment(texture, {
    dimension: '2d',
    baseArrayLayer: layer1,
    baseMipLevel: level1,
    mipLevelCount: 1
  });
  const encoder = t.device.createCommandEncoder();
  if (inSamePass) {
    const renderPass = encoder.beginRenderPass({
      colorAttachments: [colorAttachment1, colorAttachment2]
    });
    renderPass.end();
  } else {
    const renderPass1 = encoder.beginRenderPass({
      colorAttachments: [colorAttachment1]
    });
    renderPass1.end();
    const renderPass2 = encoder.beginRenderPass({
      colorAttachments: [colorAttachment2]
    });
    renderPass2.end();
  }

  const success = inSamePass ? layer0 !== layer1 : true;
  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

g.test('subresources,color_attachment_and_bind_group').
desc(
  `
  Test that when one subresource of a texture is used as a color attachment, it cannot be used in a
  bind group simultaneously in the same render pass encoder. It is allowed when the bind group is
  used in another render pass encoder instead of the same one.`
).
params((u) =>
u.
combine('colorAttachmentLevel', [0, 1]).
combine('colorAttachmentLayer', [0, 1]).
combineWithParams([
{ bgLevel: 0, bgLevelCount: 1 },
{ bgLevel: 1, bgLevelCount: 1 },
{ bgLevel: 1, bgLevelCount: 2 }]
).
combineWithParams([
{ bgLayer: 0, bgLayerCount: 1 },
{ bgLayer: 1, bgLayerCount: 1 },
{ bgLayer: 1, bgLayerCount: 2 }]
).
combine('bgUsage', kTextureBindingTypes).
unless((t) => t.bgUsage !== 'sampled-texture' && t.bgLevelCount > 1).
combine('inSamePass', [true, false])
).
fn((t) => {
  const {
    colorAttachmentLevel,
    colorAttachmentLayer,
    bgLevel,
    bgLevelCount,
    bgLayer,
    bgLayerCount,
    bgUsage,
    inSamePass
  } = t.params;

  const texture = t.createTextureTracked({
    format: 'r32float',
    usage:
    GPUTextureUsage.RENDER_ATTACHMENT |
    GPUTextureUsage.TEXTURE_BINDING |
    GPUTextureUsage.STORAGE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers],
    mipLevelCount: kTextureLevels
  });
  const bindGroupView = texture.createView({
    dimension: '2d-array',
    baseArrayLayer: bgLayer,
    arrayLayerCount: bgLayerCount,
    baseMipLevel: bgLevel,
    mipLevelCount: bgLevelCount
  });
  const bindGroup = t.createBindGroupForTest(bindGroupView, bgUsage, 'unfilterable-float');

  const colorAttachment = t.getColorAttachment(texture, {
    dimension: '2d',
    baseArrayLayer: colorAttachmentLayer,
    arrayLayerCount: 1,
    baseMipLevel: colorAttachmentLevel,
    mipLevelCount: 1
  });

  const encoder = t.device.createCommandEncoder();
  const renderPass = encoder.beginRenderPass({
    colorAttachments: [colorAttachment]
  });
  if (inSamePass) {
    renderPass.setBindGroup(0, bindGroup);
    renderPass.end();
  } else {
    renderPass.end();

    const texture2 = t.createTextureTracked({
      format: 'r32float',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      size: [kTextureSize, kTextureSize, 1],
      mipLevelCount: 1
    });
    const colorAttachment2 = t.getColorAttachment(texture2);
    const renderPass2 = encoder.beginRenderPass({
      colorAttachments: [colorAttachment2]
    });
    renderPass2.setBindGroup(0, bindGroup);
    renderPass2.end();
  }

  const isMipLevelNotOverlapped = t.isRangeNotOverlapped(
    colorAttachmentLevel,
    colorAttachmentLevel,
    bgLevel,
    bgLevel + bgLevelCount - 1
  );
  const isArrayLayerNotOverlapped = t.isRangeNotOverlapped(
    colorAttachmentLayer,
    colorAttachmentLayer,
    bgLayer,
    bgLayer + bgLayerCount - 1
  );
  const isNotOverlapped = isMipLevelNotOverlapped || isArrayLayerNotOverlapped;

  const success = inSamePass ? isNotOverlapped : true;
  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

g.test('subresources,depth_stencil_attachment_and_bind_group').
desc(
  `
  Test that when one subresource of a texture is used as a depth stencil attachment, it cannot be
  used in a bind group simultaneously in the same render pass encoder. It is allowed when the bind
  group is used in another render pass encoder instead of the same one, or the subresource is used
  as a read-only depth stencil attachment.`
).
params((u) =>
u.
combine('dsLevel', [0, 1]).
combine('dsLayer', [0, 1]).
combineWithParams([
{ bgLevel: 0, bgLevelCount: 1 },
{ bgLevel: 1, bgLevelCount: 1 },
{ bgLevel: 1, bgLevelCount: 2 }]
).
combineWithParams([
{ bgLayer: 0, bgLayerCount: 1 },
{ bgLayer: 1, bgLayerCount: 1 },
{ bgLayer: 1, bgLayerCount: 2 }]
).
beginSubcases().
combine('depthReadOnly', [true, false]).
combine('stencilReadOnly', [true, false]).
combine('bgAspect', ['depth-only', 'stencil-only']).
combine('inSamePass', [true, false])
).
fn((t) => {
  const {
    dsLevel,
    dsLayer,
    bgLevel,
    bgLevelCount,
    bgLayer,
    bgLayerCount,
    depthReadOnly,
    stencilReadOnly,
    bgAspect,
    inSamePass
  } = t.params;

  const texture = t.createTextureTracked({
    format: 'depth24plus-stencil8',
    usage: GPUTextureUsage.RENDER_ATTACHMENT | GPUTextureUsage.TEXTURE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers],
    mipLevelCount: kTextureLevels
  });
  const bindGroupView = texture.createView({
    dimension: '2d-array',
    baseArrayLayer: bgLayer,
    arrayLayerCount: bgLayerCount,
    baseMipLevel: bgLevel,
    mipLevelCount: bgLevelCount,
    aspect: bgAspect
  });
  const sampleType = bgAspect === 'depth-only' ? 'depth' : 'uint';
  const bindGroup = t.createBindGroupForTest(bindGroupView, 'sampled-texture', sampleType);

  const attachmentView = texture.createView({
    dimension: '2d',
    baseArrayLayer: dsLayer,
    arrayLayerCount: 1,
    baseMipLevel: dsLevel,
    mipLevelCount: 1
  });
  const depthStencilAttachment = {
    view: attachmentView,
    depthReadOnly,
    depthLoadOp: depthReadOnly ? undefined : 'load',
    depthStoreOp: depthReadOnly ? undefined : 'store',
    stencilReadOnly,
    stencilLoadOp: stencilReadOnly ? undefined : 'load',
    stencilStoreOp: stencilReadOnly ? undefined : 'store'
  };

  const encoder = t.device.createCommandEncoder();
  const renderPass = encoder.beginRenderPass({
    colorAttachments: [],
    depthStencilAttachment
  });
  if (inSamePass) {
    renderPass.setBindGroup(0, bindGroup);
    renderPass.end();
  } else {
    renderPass.end();

    const texture2 = t.createTextureTracked({
      format: 'rgba8unorm',
      usage: GPUTextureUsage.RENDER_ATTACHMENT,
      size: [kTextureSize, kTextureSize, 1],
      mipLevelCount: 1
    });
    const colorAttachment2 = t.getColorAttachment(texture2);
    const renderPass2 = encoder.beginRenderPass({
      colorAttachments: [colorAttachment2]
    });
    renderPass2.setBindGroup(0, bindGroup);
    renderPass2.end();
  }

  const isMipLevelNotOverlapped = t.isRangeNotOverlapped(
    dsLevel,
    dsLevel,
    bgLevel,
    bgLevel + bgLevelCount - 1
  );
  const isArrayLayerNotOverlapped = t.isRangeNotOverlapped(
    dsLayer,
    dsLayer,
    bgLayer,
    bgLayer + bgLayerCount - 1
  );
  const isNotOverlapped = isMipLevelNotOverlapped || isArrayLayerNotOverlapped;
  const readonly =
  bgAspect === 'stencil-only' && stencilReadOnly ||
  bgAspect === 'depth-only' && depthReadOnly;

  const success = !inSamePass || isNotOverlapped || readonly;
  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

g.test('subresources,multiple_bind_groups').
desc(
  `
  Test that when one color texture subresource is bound to different bind groups, its list of
  internal usages within one usage scope can only be a compatible usage list. For texture
  subresources in bind groups, the compatible usage lists are {TEXTURE_BINDING} and
  {STORAGE_BINDING}, which means it can only be bound as both TEXTURE_BINDING and STORAGE_BINDING in
  different render pass encoders, otherwise a validation error will occur.`
).
params((u) =>
u.
combine('bg0Levels', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('bg0Layers', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('bg1Levels', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('bg1Layers', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('bgUsage0', kTextureBindingTypes).
combine('bgUsage1', kTextureBindingTypes).
unless(
  (t) =>
  t.bgUsage0 !== 'sampled-texture' && t.bg0Levels.count > 1 ||
  t.bgUsage1 !== 'sampled-texture' && t.bg1Levels.count > 1
).
beginSubcases().
combine('inSamePass', [true, false])
).
fn((t) => {
  const { bg0Levels, bg0Layers, bg1Levels, bg1Layers, bgUsage0, bgUsage1, inSamePass } = t.params;

  const texture = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.STORAGE_BINDING | GPUTextureUsage.TEXTURE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers],
    mipLevelCount: kTextureLevels
  });
  const bg0 = texture.createView({
    dimension: '2d-array',
    baseArrayLayer: bg0Layers.base,
    arrayLayerCount: bg0Layers.count,
    baseMipLevel: bg0Levels.base,
    mipLevelCount: bg0Levels.count
  });
  const bg1 = texture.createView({
    dimension: '2d-array',
    baseArrayLayer: bg1Layers.base,
    arrayLayerCount: bg1Layers.count,
    baseMipLevel: bg1Levels.base,
    mipLevelCount: bg1Levels.count
  });
  const bindGroup0 = t.createBindGroupForTest(bg0, bgUsage0, 'unfilterable-float');
  const bindGroup1 = t.createBindGroupForTest(bg1, bgUsage1, 'unfilterable-float');

  const colorTexture = t.createTextureTracked({
    format: 'r32float',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1],
    mipLevelCount: 1
  });
  const colorAttachment = t.getColorAttachment(colorTexture);
  const encoder = t.device.createCommandEncoder();
  const renderPass = encoder.beginRenderPass({
    colorAttachments: [colorAttachment]
  });
  if (inSamePass) {
    renderPass.setBindGroup(0, bindGroup0);
    renderPass.setBindGroup(1, bindGroup1);
    renderPass.end();
  } else {
    renderPass.setBindGroup(0, bindGroup0);
    renderPass.end();

    const renderPass2 = encoder.beginRenderPass({
      colorAttachments: [colorAttachment]
    });
    renderPass2.setBindGroup(1, bindGroup1);
    renderPass2.end();
  }

  const bothReadOnly =
  IsReadOnlyTextureBindingType(bgUsage0) && IsReadOnlyTextureBindingType(bgUsage1);
  const isMipLevelNotOverlapped = t.isRangeNotOverlapped(
    bg0Levels.base,
    bg0Levels.base + bg0Levels.count - 1,
    bg1Levels.base,
    bg1Levels.base + bg1Levels.count - 1
  );
  const isArrayLayerNotOverlapped = t.isRangeNotOverlapped(
    bg0Layers.base,
    bg0Layers.base + bg0Layers.count - 1,
    bg1Layers.base,
    bg1Layers.base + bg1Layers.count - 1
  );
  const isNotOverlapped = isMipLevelNotOverlapped || isArrayLayerNotOverlapped;

  const success = !inSamePass || bothReadOnly || isNotOverlapped || bgUsage0 === bgUsage1;
  t.expectValidationError(() => {
    encoder.finish();
  }, !success);
});

g.test('subresources,depth_stencil_texture_in_bind_groups').
desc(
  `
  Test that when one depth stencil texture subresource is bound to different bind groups, we can
  always bind these two bind groups in either the same or different render pass encoder as the depth
  stencil texture can only be bound as TEXTURE_BINDING in the bind group.`
).
params((u) =>
u.
combine('view0Levels', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('view0Layers', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('view1Levels', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('view1Layers', [
{ base: 0, count: 1 },
{ base: 1, count: 1 },
{ base: 1, count: 2 }]
).
combine('aspect0', ['depth-only', 'stencil-only']).
combine('aspect1', ['depth-only', 'stencil-only']).
combine('inSamePass', [true, false])
).
fn((t) => {
  const { view0Levels, view0Layers, view1Levels, view1Layers, aspect0, aspect1, inSamePass } =
  t.params;

  const texture = t.createTextureTracked({
    format: 'depth24plus-stencil8',
    usage: GPUTextureUsage.TEXTURE_BINDING,
    size: [kTextureSize, kTextureSize, kTextureLayers],
    mipLevelCount: kTextureLevels
  });
  const bindGroupView0 = texture.createView({
    dimension: '2d-array',
    baseArrayLayer: view0Layers.base,
    arrayLayerCount: view0Layers.count,
    baseMipLevel: view0Levels.base,
    mipLevelCount: view0Levels.count,
    aspect: aspect0
  });
  const bindGroupView1 = texture.createView({
    dimension: '2d-array',
    baseArrayLayer: view1Layers.base,
    arrayLayerCount: view1Layers.count,
    baseMipLevel: view1Levels.base,
    mipLevelCount: view1Levels.count,
    aspect: aspect1
  });

  const sampleType0 = aspect0 === 'depth-only' ? 'depth' : 'uint';
  const sampleType1 = aspect1 === 'depth-only' ? 'depth' : 'uint';
  const bindGroup0 = t.createBindGroupForTest(bindGroupView0, 'sampled-texture', sampleType0);
  const bindGroup1 = t.createBindGroupForTest(bindGroupView1, 'sampled-texture', sampleType1);

  const colorTexture = t.createTextureTracked({
    format: 'rgba8unorm',
    usage: GPUTextureUsage.RENDER_ATTACHMENT,
    size: [kTextureSize, kTextureSize, 1],
    mipLevelCount: 1
  });
  const colorAttachment = t.getColorAttachment(colorTexture);
  const encoder = t.device.createCommandEncoder();
  const renderPass = encoder.beginRenderPass({
    colorAttachments: [colorAttachment]
  });
  if (inSamePass) {
    renderPass.setBindGroup(0, bindGroup0);
    renderPass.setBindGroup(1, bindGroup1);
    renderPass.end();
  } else {
    renderPass.setBindGroup(0, bindGroup0);
    renderPass.end();

    const renderPass2 = encoder.beginRenderPass({
      colorAttachments: [colorAttachment]
    });
    renderPass2.setBindGroup(1, bindGroup1);
    renderPass2.end();
  }

  t.expectValidationError(() => {
    encoder.finish();
  }, false);
});