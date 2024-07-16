/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for the readonly_and_readwrite_storage_textures language feature
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { TexelFormats } from '../../types.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kFeatureName = 'readonly_and_readwrite_storage_textures';

g.test('var_decl').
desc(
  `Checks that the read and read_write access modes are only allowed with the language feature present`
).
paramsSubcasesOnly((u) =>
u.
combine('type', [
'texture_storage_1d',
'texture_storage_2d',
'texture_storage_2d_array',
'texture_storage_3d']
).
combine('format', TexelFormats).
combine('access', ['read', 'write', 'read_write'])
).
fn((t) => {
  const { type, format, access } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(format.format);

  const source = `@group(0) @binding(0) var t : ${type}<${format.format}, ${access}>;`;
  const requiresFeature = access !== 'write';
  t.expectCompileResult(t.hasLanguageFeature(kFeatureName) || !requiresFeature, source);
});

g.test('textureBarrier').
desc(
  `Checks that the textureBarrier() builtin is only allowed with the language feature present`
).
fn((t) => {
  t.expectCompileResult(
    t.hasLanguageFeature(kFeatureName),
    `
        @workgroup_size(1) @compute fn main() {
            textureBarrier();
        }
    `
  );
});