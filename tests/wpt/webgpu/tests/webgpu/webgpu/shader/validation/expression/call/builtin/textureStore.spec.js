/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureStore';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureStore coords parameter must be correct type
* test textureStore array_index parameter must be correct type
* test textureStore value parameter must be correct type
* test textureStore doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { kAllTextureFormats, kTextureFormatInfo } from '../../../../../format_info.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,


  isUnsignedType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  getSampleAndBaseTextureTypeForTextureType,
  kTestTextureTypes } from
'./shader_builtin_utils.js';

const kTextureColorTypeToType = {
  sint: Type.vec4i,
  uint: Type.vec4u,
  float: Type.vec4f,
  'unfilterable-float': Type.vec4f
};






const kValidTextureStoreParameterTypes = {
  texture_storage_1d: { coordsArgTypes: [Type.i32, Type.u32] },
  texture_storage_2d: { coordsArgTypes: [Type.vec2i, Type.vec2u] },
  texture_storage_2d_array: {
    coordsArgTypes: [Type.vec2i, Type.vec2u],
    hasArrayIndexArg: true
  },
  texture_storage_3d: { coordsArgTypes: [Type.vec3i, Type.vec3u] }
};

const kTextureTypes = keysOf(kValidTextureStoreParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturestore').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureStoreParameterTypes)).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0)
).
fn((t) => {
  const { textureType, coordType, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const { coordsArgTypes, hasArrayIndexArg } = kValidTextureStoreParameterTypes[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const format = 'rgba8unorm';
  const valueWGSL = 'vec4f(0)';

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format},write>;
@fragment fn fs() -> @location(0) vec4f {
  textureStore(t, ${coordWGSL}${arrayWGSL}, ${valueWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(coordArgType, coordsArgTypes[0]) ||
  isConvertible(coordArgType, coordsArgTypes[1]);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturestore').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureStoreParameterTypes[t.textureType].hasArrayIndexArg).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0)
).
fn((t) => {
  const { textureType, arrayIndexType, value } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgTypes } = kValidTextureStoreParameterTypes[textureType];

  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const format = 'rgba8unorm';
  const valueWGSL = 'vec4f(0)';

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format}, write>;
@fragment fn fs() -> @location(0) vec4f {
  textureStore(t, ${coordWGSL}, ${arrayWGSL}, ${valueWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('value_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturestore').
desc(
  `
Validates that only incorrect value arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('valueType', keysOf(kValuesTypes)).
beginSubcases().
combine('format', kAllTextureFormats)
// filter to only storage texture formats.
.filter((t) => !!kTextureFormatInfo[t.format].color?.storage).
combine('value', [0, 1, 2])
).
fn((t) => {
  const { textureType, valueType, format, value } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(format);

  const valueArgType = kValuesTypes[valueType];
  const args = [valueArgType.create(value)];
  const { coordsArgTypes, hasArrayIndexArg } = kValidTextureStoreParameterTypes[textureType];

  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const valueWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format}, write>;
@fragment fn fs() -> @location(0) vec4f {
  textureStore(t, ${coordWGSL}${arrayWGSL}, ${valueWGSL});
  return vec4f(0);
}
`;
  const colorType = kTextureFormatInfo[format].color?.type;
  const requiredValueType = kTextureColorTypeToType[colorType];
  const expectSuccess = isConvertible(valueArgType, requiredValueType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type,storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturestore').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
beginSubcases().
combine('textureType', keysOf(kValidTextureStoreParameterTypes)).
combine('format', kAllTextureFormats)
// filter to only storage texture formats.
.filter((t) => !!kTextureFormatInfo[t.format].color?.storage)
).
fn((t) => {
  const { testTextureType, textureType, format } = t.params;
  const { coordsArgTypes, hasArrayIndexArg } = kValidTextureStoreParameterTypes[textureType];

  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const colorType = kTextureFormatInfo[format].color?.type;
  const valueType = kTextureColorTypeToType[colorType];
  const valueWGSL = valueType.create(0).wgsl();

  const code = `
@group(0) @binding(1) var t: ${testTextureType.replace(', read', ', write')};
@fragment fn fs() -> @location(0) vec4f {
  textureStore(t, ${coordWGSL}${arrayWGSL}, ${valueWGSL});
  return vec4f(0);
}
`;

  const [baseTestTextureType, sampleType] =
  getSampleAndBaseTextureTypeForTextureType(testTextureType);

  let expectSuccess = false;
  const types = kValidTextureStoreParameterTypes[baseTestTextureType];
  if (types) {
    const typesMatch = types ?
    types.coordsArgTypes[0] === coordsArgTypes[0] &&
    types.hasArrayIndexArg === hasArrayIndexArg &&
    isConvertible(valueType, sampleType) :
    false;
    expectSuccess = typesMatch;
  }

  t.expectCompileResult(expectSuccess, code);
});