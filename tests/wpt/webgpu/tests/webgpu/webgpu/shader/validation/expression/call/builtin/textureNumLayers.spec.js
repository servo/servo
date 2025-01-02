/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureNumLayers';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureNumLayers returns the correct type
* test textureNumLayers doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { kAllTextureFormats, kTextureFormatInfo } from '../../../../../format_info.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,
  stringToType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  getNonStorageTextureTypeWGSL,
  kNonStorageTextureTypeInfo,
  kTestTextureTypes } from
'./shader_builtin_utils.js';

const kTextureNumLayersTextureTypesForNonStorageTextures = [
'texture_2d_array',
'texture_cube_array',
'texture_depth_2d_array',
'texture_depth_cube_array'];


const kTextureNumLayersTextureTypesForStorageTextures = ['texture_storage_2d_array'];

const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturenumlayers').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', kTextureNumLayersTextureTypesForNonStorageTextures).
beginSubcases().
expand('texelType', (t) =>
kNonStorageTextureTypeInfo[t.textureType].texelTypes.map((v) => v.toString())
)
).
fn((t) => {
  const { returnType, textureType, texelType } = t.params;
  const returnVarType = kValuesTypes[returnType];

  const varWGSL = returnVarType.toString();
  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureNumLayers(t);
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.u32, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('return_type,storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturenumlayers').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', kTextureNumLayersTextureTypesForStorageTextures).
beginSubcases().
combine('format', kAllTextureFormats)
// filter to only storage texture formats.
.filter((t) => !!kTextureFormatInfo[t.format].color?.storage)
).
fn((t) => {
  const { returnType, textureType, format } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(format);

  const returnVarType = kValuesTypes[returnType];

  const varWGSL = returnVarType.toString();

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format}, read>;
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureNumLayers(t);
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.u32, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturenumlayers').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) => u.combine('testTextureType', kTestTextureTypes)).
fn((t) => {
  const { testTextureType } = t.params;
  const code = `
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureNumLayers(t);
  return vec4f(0);
}
`;

  const expectSuccess = testTextureType.includes('array');

  t.expectCompileResult(expectSuccess, code);
});

g.test('must_use').
desc('Tests that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var t : texture_2d_array<f32>;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureNumLayers(t);
    }`;
  t.expectCompileResult(t.params.use, code);
});