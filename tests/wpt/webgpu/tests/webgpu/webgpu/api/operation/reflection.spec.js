/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `
Tests that object attributes which reflect the object's creation properties are properly set.
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUConst } from '../../constants.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

g.test('buffer_reflection_attributes')
  .desc(`For every buffer attribute, the corresponding descriptor value is carried over.`)
  .paramsSubcasesOnly(u =>
    u.combine('descriptor', [
      { size: 4, usage: GPUConst.BufferUsage.VERTEX },
      {
        size: 16,
        usage:
          GPUConst.BufferUsage.STORAGE |
          GPUConst.BufferUsage.COPY_SRC |
          GPUConst.BufferUsage.UNIFORM,
      },
      { size: 32, usage: GPUConst.BufferUsage.MAP_READ | GPUConst.BufferUsage.COPY_DST },
      {
        size: 32,
        usage: GPUConst.BufferUsage.MAP_READ | GPUConst.BufferUsage.MAP_WRITE,
        invalid: true,
      },
    ])
  )
  .fn(t => {
    const { descriptor } = t.params;

    t.expectValidationError(() => {
      const buffer = t.device.createBuffer(descriptor);

      t.expect(buffer.size === descriptor.size);
      t.expect(buffer.usage === descriptor.usage);
    }, descriptor.invalid === true);
  });

g.test('texture_reflection_attributes')
  .desc(`For every texture attribute, the corresponding descriptor value is carried over.`)
  .paramsSubcasesOnly(u =>
    u.combine('descriptor', [
      {
        size: { width: 4, height: 4 },
        format: 'rgba8unorm',
        usage: GPUConst.TextureUsage.TEXTURE_BINDING,
      },
      {
        size: { width: 8, height: 8, depthOrArrayLayers: 8 },
        format: 'bgra8unorm',
        usage: GPUConst.TextureUsage.RENDER_ATTACHMENT | GPUConst.TextureUsage.COPY_SRC,
      },
      {
        size: [4, 4],
        format: 'rgba8unorm',
        usage: GPUConst.TextureUsage.TEXTURE_BINDING,
        mipLevelCount: 2,
      },
      {
        size: [16, 16, 16],
        format: 'rgba8unorm',
        usage: GPUConst.TextureUsage.TEXTURE_BINDING,
        dimension: '3d',
      },
      {
        size: [32],
        format: 'rgba8unorm',
        usage: GPUConst.TextureUsage.TEXTURE_BINDING,
        dimension: '1d',
      },
      {
        size: { width: 4, height: 4 },
        format: 'rgba8unorm',
        usage: GPUConst.TextureUsage.RENDER_ATTACHMENT,
        sampleCount: 4,
      },
      {
        size: { width: 4, height: 4 },
        format: 'rgba8unorm',
        usage: GPUConst.TextureUsage.TEXTURE_BINDING,
        sampleCount: 4,
        invalid: true,
      },
    ])
  )
  .fn(t => {
    const { descriptor } = t.params;

    let width;
    let height;
    let depthOrArrayLayers;
    if (Array.isArray(descriptor.size)) {
      width = descriptor.size[0];
      height = descriptor.size[1] || 1;
      depthOrArrayLayers = descriptor.size[2] || 1;
    } else {
      width = descriptor.size.width;
      height = descriptor.size.height || 1;
      depthOrArrayLayers = descriptor.size.depthOrArrayLayers || 1;
    }

    t.expectValidationError(() => {
      const texture = t.device.createTexture(descriptor);

      t.expect(texture.width === width);
      t.expect(texture.height === height);
      t.expect(texture.depthOrArrayLayers === depthOrArrayLayers);
      t.expect(texture.format === descriptor.format);
      t.expect(texture.usage === descriptor.usage);
      t.expect(texture.dimension === (descriptor.dimension || '2d'));
      t.expect(texture.mipLevelCount === (descriptor.mipLevelCount || 1));
      t.expect(texture.sampleCount === (descriptor.sampleCount || 1));
    }, descriptor.invalid === true);
  });

g.test('query_set_reflection_attributes')
  .desc(`For every queue attribute, the corresponding descriptor value is carried over.`)
  .paramsSubcasesOnly(u =>
    u.combine('descriptor', [
      { type: 'occlusion', count: 4 },
      { type: 'occlusion', count: 16 },
      { type: 'occlusion', count: 8193, invalid: true },
    ])
  )
  .fn(t => {
    const { descriptor } = t.params;

    t.expectValidationError(() => {
      const querySet = t.device.createQuerySet(descriptor);

      t.expect(querySet.type === descriptor.type);
      t.expect(querySet.count === descriptor.count);
    }, descriptor.invalid === true);
  });
