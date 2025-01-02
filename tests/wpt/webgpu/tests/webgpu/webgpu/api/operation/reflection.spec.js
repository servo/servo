/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Tests that object attributes which reflect the object's creation properties are properly set.
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { GPUConst } from '../../constants.js';
import { GPUTest } from '../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

function* extractValuePropertyKeys(obj) {
  for (const key in obj) {
    if (typeof obj[key] !== 'function') {
      yield key;
    }
  }
}

const kBufferSubcases =




[
{ size: 4, usage: GPUConst.BufferUsage.VERTEX },
{
  size: 16,
  usage:
  GPUConst.BufferUsage.STORAGE | GPUConst.BufferUsage.COPY_SRC | GPUConst.BufferUsage.UNIFORM
},
{ size: 32, usage: GPUConst.BufferUsage.MAP_READ | GPUConst.BufferUsage.COPY_DST },
{ size: 40, usage: GPUConst.BufferUsage.INDEX, label: 'some label' },
{
  size: 32,
  usage: GPUConst.BufferUsage.MAP_READ | GPUConst.BufferUsage.MAP_WRITE,
  invalid: true
}];


g.test('buffer_reflection_attributes').
desc(`For every buffer attribute, the corresponding descriptor value is carried over.`).
paramsSubcasesOnly((u) => u.combine('descriptor', kBufferSubcases)).
fn((t) => {
  const { descriptor } = t.params;

  t.expectValidationError(() => {
    const buffer = t.createBufferTracked(descriptor);

    t.expect(buffer.size === descriptor.size);
    t.expect(buffer.usage === descriptor.usage);
  }, descriptor.invalid === true);
});

g.test('buffer_creation_from_reflection').
desc(
  `
    Check that you can create a buffer from a buffer's reflection.
    This check is to insure that as WebGPU develops this path doesn't
    suddenly break because of new reflection.
  `
).
paramsSubcasesOnly((u) =>
u.combine('descriptor', kBufferSubcases).filter((p) => !p.descriptor.invalid)
).

fn((t) => {
  const { descriptor } = t.params;

  const buffer = t.createBufferTracked(descriptor);
  const buffer2 = t.createBufferTracked(buffer);

  const bufferAsObject = buffer;
  const buffer2AsObject = buffer2;
  const keys = [...extractValuePropertyKeys(bufferAsObject)];

  // Sanity check
  t.expect(keys.includes('size'));
  t.expect(keys.includes('usage'));
  t.expect(keys.includes('label'));

  for (const key of keys) {
    t.expect(bufferAsObject[key] === buffer2AsObject[key], key);
  }
});

const kTextureSubcases =








[
{
  size: { width: 4, height: 4 },
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.TEXTURE_BINDING
},
{
  size: { width: 4, height: 4 },
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.TEXTURE_BINDING,
  label: 'some label'
},
{
  size: { width: 8, height: 8, depthOrArrayLayers: 8 },
  format: 'bgra8unorm',
  usage: GPUConst.TextureUsage.RENDER_ATTACHMENT | GPUConst.TextureUsage.COPY_SRC
},
{
  size: [4, 4],
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.TEXTURE_BINDING,
  mipLevelCount: 2
},
{
  size: [16, 16, 16],
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.TEXTURE_BINDING,
  dimension: '3d'
},
{
  size: [32],
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.TEXTURE_BINDING,
  dimension: '1d'
},
{
  size: { width: 4, height: 4 },
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.RENDER_ATTACHMENT,
  sampleCount: 4
},
{
  size: { width: 4, height: 4 },
  format: 'rgba8unorm',
  usage: GPUConst.TextureUsage.TEXTURE_BINDING,
  sampleCount: 4,
  invalid: true
}];


g.test('texture_reflection_attributes').
desc(`For every texture attribute, the corresponding descriptor value is carried over.`).
paramsSubcasesOnly((u) => u.combine('descriptor', kTextureSubcases)).
fn((t) => {
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
    const texture = t.createTextureTracked(descriptor);

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





g.test('texture_creation_from_reflection').
desc(
  `
    Check that you can create a texture from a texture's reflection.
    This check is to insure that as WebGPU develops this path doesn't
    suddenly break because of new reflection.
  `
).
paramsSubcasesOnly((u) =>
u.combine('descriptor', kTextureSubcases).filter((p) => !p.descriptor.invalid)
).
fn((t) => {
  const { descriptor } = t.params;

  const texture = t.createTextureTracked(descriptor);
  const textureWithSize = texture;
  textureWithSize.size = [texture.width, texture.height, texture.depthOrArrayLayers];
  const texture2 = t.createTextureTracked(textureWithSize);

  const textureAsObject = texture;
  const texture2AsObject = texture2;
  const keys = [...extractValuePropertyKeys(textureAsObject)].filter((k) => k !== 'size');

  // Sanity check
  t.expect(keys.includes('format'));
  t.expect(keys.includes('usage'));
  t.expect(keys.includes('label'));

  for (const key of keys) {
    t.expect(textureAsObject[key] === texture2AsObject[key], key);
  }

  // MAINTENANCE_TODO: Check this if it is made possible by a spec change.
  //
  //     texture3 = t.createTextureTracked({
  //       ...texture,
  //       size: [texture.width, texture.height, texture.depthOrArrayLayers],
  //     });
  //
  // and this
  //
  //     texture3 = t.createTextureTracked({
  //       size: [texture.width, texture.height, texture.depthOrArrayLayers],
  //       ...texture,
  //     });
});

const kQuerySetSubcases =




[
{ type: 'occlusion', count: 4 },
{ type: 'occlusion', count: 16 },
{ type: 'occlusion', count: 32, label: 'some label' },
{ type: 'occlusion', count: 8193, invalid: true }];


g.test('query_set_reflection_attributes').
desc(`For every queue attribute, the corresponding descriptor value is carried over.`).
paramsSubcasesOnly((u) => u.combine('descriptor', kQuerySetSubcases)).
fn((t) => {
  const { descriptor } = t.params;

  t.expectValidationError(() => {
    const querySet = t.createQuerySetTracked(descriptor);

    t.expect(querySet.type === descriptor.type);
    t.expect(querySet.count === descriptor.count);
  }, descriptor.invalid === true);
});

g.test('query_set_creation_from_reflection').
desc(
  `
    Check that you can create a queryset from a queryset's reflection.
    This check is to insure that as WebGPU develops this path doesn't
    suddenly break because of new reflection.
  `
).
paramsSubcasesOnly((u) =>
u.combine('descriptor', kQuerySetSubcases).filter((p) => !p.descriptor.invalid)
).
fn((t) => {
  const { descriptor } = t.params;

  const querySet = t.createQuerySetTracked(descriptor);
  const querySet2 = t.createQuerySetTracked(querySet);

  const querySetAsObject = querySet;
  const querySet2AsObject = querySet2;
  const keys = [...extractValuePropertyKeys(querySetAsObject)];

  // Sanity check
  t.expect(keys.includes('type'));
  t.expect(keys.includes('count'));
  t.expect(keys.includes('label'));

  for (const key of keys) {
    t.expect(querySetAsObject[key] === querySet2AsObject[key], key);
  }
});