/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/

export const description = `
Test the values of flags interfaces (e.g. GPUTextureUsage).
`;
import { makeTestGroup } from '../../../common/framework/test_group.js';
import { IDLTest } from '../idl_test.js';
export const g = makeTestGroup(IDLTest);
g.test('BufferUsage').fn(t => {
  const expected = {
    MAP_READ: 0x0001,
    MAP_WRITE: 0x0002,
    COPY_SRC: 0x0004,
    COPY_DST: 0x0008,
    INDEX: 0x0010,
    VERTEX: 0x0020,
    UNIFORM: 0x0040,
    STORAGE: 0x0080,
    INDIRECT: 0x0100
  };
  t.assertMembers(GPUBufferUsage, expected);
});
g.test('TextureUsage').fn(t => {
  const expected = {
    COPY_SRC: 0x01,
    COPY_DST: 0x02,
    SAMPLED: 0x04,
    STORAGE: 0x08,
    OUTPUT_ATTACHMENT: 0x10
  };
  t.assertMembers(GPUTextureUsage, expected);
});
g.test('ColorWrite').fn(t => {
  const expected = {
    RED: 0x1,
    GREEN: 0x2,
    BLUE: 0x4,
    ALPHA: 0x8,
    ALL: 0xf
  };
  t.assertMembers(GPUColorWrite, expected);
});
g.test('ShaderStage').fn(t => {
  const expected = {
    VERTEX: 0x1,
    FRAGMENT: 0x2,
    COMPUTE: 0x4
  };
  t.assertMembers(GPUShaderStage, expected);
});
//# sourceMappingURL=flags.spec.js.map