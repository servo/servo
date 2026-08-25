/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for the readonly_and_readwrite_storage_textures language feature
`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { hasFeature } from '../../../../common/util/util.js';
import {
  kPossibleStorageTextureFormats,
  kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly } from
'../../../format_info.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kAccessModeFeatureName = 'readonly_and_readwrite_storage_textures';
const kTier1FeatureName = 'texture_formats_tier1';
const kTier1DeviceFeatureName = 'texture-formats-tier1';

g.test('var_decl').
desc(
  `Checks that the read and read_write access modes are only allowed with the language feature present

    TODO(https://github.com/gpuweb/cts/issues/4612): Stop checking the device feature
    `
).
paramsSubcasesOnly((u) =>
u.
combine('type', [
'texture_storage_1d',
'texture_storage_2d',
'texture_storage_2d_array',
'texture_storage_3d']
).
combine('format', kPossibleStorageTextureFormats).
combine('access', ['read', 'write', 'read_write'])
).
fn((t) => {
  const { type, format, access } = t.params;

  let valid = true;
  if (access !== 'write') {
    valid &&= t.hasLanguageFeature(kAccessModeFeatureName);
  }

  if (kTextureFormatsTier1EnablesStorageReadOnlyWriteOnly.indexOf(format) >= 0) {
    // Even though some of these formats do not support read-write access
    // without the tier2 adapter feature, their validity in WGSL should
    // depend only on the language feature for tier1.

    // However, because the language feature is new, also check the device
    // feature. MAINTENANCE_TODO(https://github.com/gpuweb/cts/issues/4612):
    // Stop doing this; make `if` body unconditional
    if (!hasFeature(t.device.features, kTier1DeviceFeatureName)) {
      valid &&= t.hasLanguageFeature(kTier1FeatureName);
    }
  }

  const source = `@group(0) @binding(0) var t : ${type}<${format}, ${access}>;`;
  t.expectCompileResult(valid, source);
});

g.test('textureBarrier').
desc(
  `Checks that the textureBarrier() builtin is only allowed with the language feature present`
).
fn((t) => {
  t.expectCompileResult(
    t.hasLanguageFeature(kAccessModeFeatureName),
    `
        @workgroup_size(1) @compute fn main() {
            textureBarrier();
        }
    `
  );
});