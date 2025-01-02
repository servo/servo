/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureLoad';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureLoad coords parameter must be correct type
* test textureLoad array_index parameter must be correct type
* test textureLoad level parameter must be correct type
* test textureLoad sample_index parameter must be correct type
* test textureLoad returns the correct type
* test textureLoad doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import { assert } from '../../../../../../common/util/util.js';
import { kAllTextureFormats, kTextureFormatInfo } from '../../../../../format_info.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,


  isUnsignedType,
  stringToType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import {
  getNonStorageTextureTypeWGSL,
  getSampleAndBaseTextureTypeForTextureType,
  kNonStorageTextureTypeInfo,
  kTestTextureTypes } from
'./shader_builtin_utils.js';








const kCoords1DTypes = [Type.i32, Type.u32];
const kCoords2DTypes = [Type.vec2i, Type.vec2u];
const kCoords3DTypes = [Type.vec3i, Type.vec3u];

const kValidTextureLoadParameterTypesForNonStorageTextures =
{
  texture_1d: {
    coordsArgTypes: kCoords1DTypes,
    hasLevelArg: true
  },
  texture_2d: {
    coordsArgTypes: kCoords2DTypes,
    hasLevelArg: true
  },
  texture_2d_array: {
    coordsArgTypes: kCoords2DTypes,
    hasArrayIndexArg: true,
    hasLevelArg: true
  },
  texture_3d: {
    coordsArgTypes: kCoords3DTypes,
    hasLevelArg: true
  },
  texture_multisampled_2d: {
    coordsArgTypes: kCoords2DTypes,
    hasSampleIndexArg: true
  },
  texture_depth_2d: {
    coordsArgTypes: kCoords2DTypes,
    hasLevelArg: true
  },
  texture_depth_2d_array: {
    coordsArgTypes: kCoords2DTypes,
    hasArrayIndexArg: true,
    hasLevelArg: true
  },
  texture_depth_multisampled_2d: {
    coordsArgTypes: kCoords2DTypes,
    hasSampleIndexArg: true
  },
  texture_external: { coordsArgTypes: kCoords2DTypes }
};

const kValidTextureLoadParameterTypesForStorageTextures = {
  texture_storage_1d: { coordsArgTypes: [Type.i32, Type.u32] },
  texture_storage_2d: { coordsArgTypes: [Type.vec2i, Type.vec2u] },
  texture_storage_2d_array: {
    coordsArgTypes: [Type.vec2i, Type.vec2u],
    hasArrayIndexArg: true
  },
  texture_storage_3d: { coordsArgTypes: [Type.vec3i, Type.vec3u] }
};

const kNonStorageTextureTypes = keysOf(kValidTextureLoadParameterTypesForNonStorageTextures);
const kStorageTextureTypes = keysOf(kValidTextureLoadParameterTypesForStorageTextures);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', kNonStorageTextureTypes).
beginSubcases().
expand('texelType', (t) =>
kNonStorageTextureTypeInfo[t.textureType].texelTypes.map((v) => v.toString())
)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.textureType)).
fn((t) => {
  const { returnType, textureType, texelType } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const { coordsArgTypes, hasArrayIndexArg, hasLevelArg, hasSampleIndexArg } =
  kValidTextureLoadParameterTypesForNonStorageTextures[textureType];

  const varWGSL = returnVarType.toString();
  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const levelWGSL = hasLevelArg ? ', 0' : '';
  const sampleIndexWGSL = hasSampleIndexArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureLoad(t, ${coordWGSL}${arrayWGSL}${levelWGSL}${sampleIndexWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(texelArgType, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kNonStorageTextureTypes).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
expand('texelType', (t) =>
kNonStorageTextureTypeInfo[t.textureType].texelTypes.map((v) => v.toString())
).
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.textureType)).
fn((t) => {
  const { textureType, coordType, texelType, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const { coordsArgTypes, hasArrayIndexArg, hasLevelArg, hasSampleIndexArg } =
  kValidTextureLoadParameterTypesForNonStorageTextures[textureType];

  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const levelWGSL = hasLevelArg ? ', 0' : '';
  const sampleIndexWGSL = hasSampleIndexArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  _ = textureLoad(t, ${coordWGSL}${arrayWGSL}${levelWGSL}${sampleIndexWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(coordArgType, coordsArgTypes[0]) ||
  isConvertible(coordArgType, coordsArgTypes[1]);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument,storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kStorageTextureTypes).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('format', kAllTextureFormats)
// filter to only storage texture formats.
.filter((t) => !!kTextureFormatInfo[t.format].color?.storage).
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0)
).
beforeAllSubcases((t) =>
t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures')
).
fn((t) => {
  const { textureType, coordType, format, value } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(format);

  const coordArgType = kValuesTypes[coordType];
  const { coordsArgTypes, hasArrayIndexArg } =
  kValidTextureLoadParameterTypesForStorageTextures[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format}, read>;
@fragment fn fs() -> @location(0) vec4f {
  _ = textureLoad(t, ${coordWGSL}${arrayWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(coordArgType, coordsArgTypes[0]) ||
  isConvertible(coordArgType, coordsArgTypes[1]);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kNonStorageTextureTypes)
// filter out types with no array_index
.filter(
  (t) => !!kValidTextureLoadParameterTypesForNonStorageTextures[t.textureType].hasArrayIndexArg
).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
expand('texelType', (t) =>
kNonStorageTextureTypeInfo[t.textureType].texelTypes.map((v) => v.toString())
).
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.textureType)).
fn((t) => {
  const { textureType, arrayIndexType, texelType, value } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgTypes, hasLevelArg } =
  kValidTextureLoadParameterTypesForNonStorageTextures[textureType];

  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const levelWGSL = hasLevelArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  _ = textureLoad(t, ${coordWGSL}, ${arrayWGSL}${levelWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument,storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kStorageTextureTypes)
// filter out types with no array_index
.filter(
  (t) => !!kValidTextureLoadParameterTypesForStorageTextures[t.textureType].hasArrayIndexArg
).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('format', kAllTextureFormats)
// filter to only storage texture formats.
.filter((t) => !!kTextureFormatInfo[t.format].color?.storage).
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0)
).
beforeAllSubcases((t) =>
t.skipIfLanguageFeatureNotSupported('readonly_and_readwrite_storage_textures')
).
fn((t) => {
  const { textureType, arrayIndexType, format, value } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(format);

  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgTypes, hasLevelArg } =
  kValidTextureLoadParameterTypesForStorageTextures[textureType];

  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const levelWGSL = hasLevelArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format}, read>;
@fragment fn fs() -> @location(0) vec4f {
  _ = textureLoad(t, ${coordWGSL}, ${arrayWGSL}${levelWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('level_argument,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that only incorrect level arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kNonStorageTextureTypes)
// filter out types with no level
.filter(
  (t) => !!kValidTextureLoadParameterTypesForNonStorageTextures[t.textureType].hasLevelArg
).
combine('levelType', keysOf(kValuesTypes)).
beginSubcases().
expand('texelType', (t) =>
kNonStorageTextureTypeInfo[t.textureType].texelTypes.map((v) => v.toString())
).
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.levelType]) || t.value >= 0)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.textureType)).
fn((t) => {
  const { textureType, levelType, texelType, value } = t.params;
  const levelArgType = kValuesTypes[levelType];
  const { coordsArgTypes, hasArrayIndexArg } =
  kValidTextureLoadParameterTypesForNonStorageTextures[textureType];

  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const levelWGSL = levelArgType.create(value).wgsl();

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  _ = textureLoad(t, ${coordWGSL}${arrayWGSL}, ${levelWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(levelArgType, Type.i32) || isConvertible(levelArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('sample_index_argument,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that only incorrect sample_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kNonStorageTextureTypes)
// filter out types with no sample_index
.filter(
  (t) => !!kValidTextureLoadParameterTypesForNonStorageTextures[t.textureType].hasSampleIndexArg
).
combine('sampleIndexType', keysOf(kValuesTypes)).
beginSubcases().
expand('texelType', (t) =>
kNonStorageTextureTypeInfo[t.textureType].texelTypes.map((v) => v.toString())
).
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.sampleIndexType]) || t.value >= 0)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.textureType)).
fn((t) => {
  const { textureType, sampleIndexType, texelType, value } = t.params;
  const sampleIndexArgType = kValuesTypes[sampleIndexType];
  const { coordsArgTypes, hasArrayIndexArg, hasLevelArg } =
  kValidTextureLoadParameterTypesForNonStorageTextures[textureType];
  assert(!hasLevelArg);

  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const sampleIndexWGSL = sampleIndexArgType.create(value).wgsl();

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  _ = textureLoad(t, ${coordWGSL}${arrayWGSL}, ${sampleIndexWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(sampleIndexArgType, Type.i32) || isConvertible(sampleIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
beginSubcases().
combine('textureType', kNonStorageTextureTypes)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.testTextureType)).
fn((t) => {
  const { testTextureType, textureType } = t.params;
  const { coordsArgTypes, hasArrayIndexArg, hasLevelArg, hasSampleIndexArg } =
  kValidTextureLoadParameterTypesForNonStorageTextures[textureType];

  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const levelWGSL = hasLevelArg ? ', 0' : '';
  const sampleIndexWGSL = hasSampleIndexArg ? ', 0' : '';

  const code = `
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureLoad(t, ${coordWGSL}${arrayWGSL}${levelWGSL}${sampleIndexWGSL});
  return vec4f(0);
}
`;

  const [baseTestTextureType] = getSampleAndBaseTextureTypeForTextureType(testTextureType);

  let expectSuccess = false;
  const types =
  kValidTextureLoadParameterTypesForNonStorageTextures[baseTestTextureType] ||
  kValidTextureLoadParameterTypesForStorageTextures[baseTestTextureType];
  if (types) {
    const numTestNumberArgs =
    (types.hasArrayIndexArg ? 1 : 0) + (
    types.hasLevelArg ? 1 : 0) + (
    types.hasSampleIndexArg ? 1 : 0);
    const numExpectNumberArgs =
    (hasArrayIndexArg ? 1 : 0) + (hasLevelArg ? 1 : 0) + (hasSampleIndexArg ? 1 : 0);
    const typesMatch = types ?
    types.coordsArgTypes[0] === coordsArgTypes[0] && numTestNumberArgs === numExpectNumberArgs :
    false;
    expectSuccess = typesMatch;
  }

  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type,storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#textureload').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
beginSubcases().
combine('textureType', kStorageTextureTypes).
combine('format', kAllTextureFormats)
).
beforeAllSubcases((t) => t.skipIfTextureLoadNotSupportedForTextureType(t.params.testTextureType)).
fn((t) => {
  const { testTextureType, textureType } = t.params;
  const { coordsArgTypes, hasArrayIndexArg, hasLevelArg, hasSampleIndexArg } =
  kValidTextureLoadParameterTypesForStorageTextures[textureType];

  const coordWGSL = coordsArgTypes[0].create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const levelWGSL = hasLevelArg ? ', 0' : '';
  const sampleIndexWGSL = hasSampleIndexArg ? ', 0' : '';

  const code = `
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureLoad(t, ${coordWGSL}${arrayWGSL}${levelWGSL}${sampleIndexWGSL});
  return vec4f(0);
}
`;

  const [baseTestTextureType] = getSampleAndBaseTextureTypeForTextureType(testTextureType);

  let expectSuccess = false;
  const types =
  kValidTextureLoadParameterTypesForNonStorageTextures[baseTestTextureType] ||
  kValidTextureLoadParameterTypesForStorageTextures[baseTestTextureType];
  if (types) {
    const numTestNumberArgs =
    (types.hasArrayIndexArg ? 1 : 0) + (
    types.hasLevelArg ? 1 : 0) + (
    types.hasSampleIndexArg ? 1 : 0);
    const numExpectNumberArgs =
    (hasArrayIndexArg ? 1 : 0) + (hasLevelArg ? 1 : 0) + (hasSampleIndexArg ? 1 : 0);
    const typesMatch = types ?
    types.coordsArgTypes[0] === coordsArgTypes[0] && numTestNumberArgs === numExpectNumberArgs :
    false;
    expectSuccess = typesMatch;
  }

  t.expectCompileResult(expectSuccess, code);
});

g.test('must_use').
desc('Tests that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var t : texture_2d<f32>;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureLoad(t, vec2(0,0), 0);
    }`;
  t.expectCompileResult(t.params.use, code);
});