/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureNumSamples';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureNumSamples returns the correct type
* test textureNumSamples doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
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

const kTextureNumSamplesTextureTypesForNonStorageTextures = [
'texture_multisampled_2d',
'texture_depth_multisampled_2d'];


const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturenumsamples').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', kTextureNumSamplesTextureTypesForNonStorageTextures).
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
  let v: ${varWGSL} = textureNumSamples(t);
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(Type.u32, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturenumsamples').
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
  let v = textureNumSamples(t);
  return vec4f(0);
}
`;
  const expectSuccess = testTextureType.includes('multisample');

  t.expectCompileResult(expectSuccess, code);
});

g.test('must_use').
desc('Tests that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var t : texture_multisampled_2d<f32>;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureDimensions(t);
    }`;
  t.expectCompileResult(t.params.use, code);
});