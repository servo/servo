/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Test the values of flags interfaces (e.g. GPUTextureUsage).
`;import { makeTestGroup } from '../../../common/framework/test_group.js';
import { IDLTest } from '../idl_test.js';

export const g = makeTestGroup(IDLTest);

const kBufferUsageExp = {
  MAP_READ: 0x0001,
  MAP_WRITE: 0x0002,
  COPY_SRC: 0x0004,
  COPY_DST: 0x0008,
  INDEX: 0x0010,
  VERTEX: 0x0020,
  UNIFORM: 0x0040,
  STORAGE: 0x0080,
  INDIRECT: 0x0100,
  QUERY_RESOLVE: 0x0200
};
g.test('BufferUsage,count').fn((t) => {
  t.assertMemberCount(GPUBufferUsage, kBufferUsageExp);
});
g.test('BufferUsage,values').
params((u) => u.combine('key', Object.keys(kBufferUsageExp))).
fn((t) => {
  const { key } = t.params;
  t.assertMember(GPUBufferUsage, kBufferUsageExp, key);
});

const kTextureUsageExp = {
  COPY_SRC: 0x01,
  COPY_DST: 0x02,
  TEXTURE_BINDING: 0x04,
  STORAGE_BINDING: 0x08,
  RENDER_ATTACHMENT: 0x10
};
g.test('TextureUsage,count').fn((t) => {
  t.assertMemberCount(GPUTextureUsage, kTextureUsageExp);
});
g.test('TextureUsage,values').
params((u) => u.combine('key', Object.keys(kTextureUsageExp))).
fn((t) => {
  const { key } = t.params;
  t.assertMember(GPUTextureUsage, kTextureUsageExp, key);
});

const kColorWriteExp = {
  RED: 0x1,
  GREEN: 0x2,
  BLUE: 0x4,
  ALPHA: 0x8,
  ALL: 0xf
};
g.test('ColorWrite,count').fn((t) => {
  t.assertMemberCount(GPUColorWrite, kColorWriteExp);
});
g.test('ColorWrite,values').
params((u) => u.combine('key', Object.keys(kColorWriteExp))).
fn((t) => {
  const { key } = t.params;
  t.assertMember(GPUColorWrite, kColorWriteExp, key);
});

const kShaderStageExp = {
  VERTEX: 0x1,
  FRAGMENT: 0x2,
  COMPUTE: 0x4
};
g.test('ShaderStage,count').fn((t) => {
  t.assertMemberCount(GPUShaderStage, kShaderStageExp);
});
g.test('ShaderStage,values').
params((u) => u.combine('key', Object.keys(kShaderStageExp))).
fn((t) => {
  const { key } = t.params;
  t.assertMember(GPUShaderStage, kShaderStageExp, key);
});