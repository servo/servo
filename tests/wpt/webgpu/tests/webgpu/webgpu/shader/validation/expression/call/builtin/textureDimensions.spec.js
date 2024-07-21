/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureDimension';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureDimension returns the correct type
* test textureDimension level parameter must be correct type
* test textureDimension doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
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






const kValidTextureDimensionParameterTypesForNonStorageTextures =

{
  texture_1d: {
    returnType: Type.u32,
    hasLevelArg: true
  },
  texture_2d: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_2d_array: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_cube: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_cube_array: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_3d: {
    returnType: Type.vec3u,
    hasLevelArg: true
  },
  texture_multisampled_2d: {
    returnType: Type.vec2u
  },
  texture_depth_2d: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_depth_2d_array: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_depth_cube: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_depth_cube_array: {
    returnType: Type.vec2u,
    hasLevelArg: true
  },
  texture_depth_multisampled_2d: {
    returnType: Type.vec2u
  },
  texture_external: { returnType: Type.vec2u }
};

const kValidTextureDimensionParameterTypesForStorageTextures =

{
  texture_storage_1d: {
    returnType: Type.u32
  },
  texture_storage_2d: {
    returnType: Type.vec2u
  },
  texture_storage_2d_array: {
    returnType: Type.vec2u
  },
  texture_storage_3d: {
    returnType: Type.vec3u
  }
};

const kNonStorageTextureTypes = keysOf(kValidTextureDimensionParameterTypesForNonStorageTextures);
const kStorageTextureTypes = keysOf(kValidTextureDimensionParameterTypesForStorageTextures);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturedimensions').
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
fn((t) => {
  const { returnType, textureType, texelType } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const { returnType: returnRequiredType, hasLevelArg } =
  kValidTextureDimensionParameterTypesForNonStorageTextures[textureType];

  const varWGSL = returnVarType.toString();
  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const levelWGSL = hasLevelArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureDimensions(t${levelWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(returnRequiredType, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('return_type,storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturedimensions').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', kStorageTextureTypes).
beginSubcases().
combine('format', kAllTextureFormats)
// filter to only storage texture formats.
.filter((t) => !!kTextureFormatInfo[t.format].color?.storage)
).
fn((t) => {
  const { returnType, textureType, format } = t.params;
  t.skipIfTextureFormatNotUsableAsStorageTexture(format);

  const returnVarType = kValuesTypes[returnType];
  const { returnType: returnRequiredType, hasLevelArg } =
  kValidTextureDimensionParameterTypesForStorageTextures[textureType];

  const varWGSL = returnVarType.toString();
  const levelWGSL = hasLevelArg ? ', 0' : '';

  const code = `
@group(0) @binding(0) var t: ${textureType}<${format}, read>;
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureDimensions(t${levelWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(returnRequiredType, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('level_argument,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturedimensions').
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
  (t) => !!kValidTextureDimensionParameterTypesForNonStorageTextures[t.textureType].hasLevelArg
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
fn((t) => {
  const { textureType, levelType, texelType, value } = t.params;
  const levelArgType = kValuesTypes[levelType];

  const texelArgType = stringToType(texelType);
  const textureWGSL = getNonStorageTextureTypeWGSL(textureType, texelArgType);
  const levelWGSL = levelArgType.create(value).wgsl();

  const code = `
@group(0) @binding(0) var t: ${textureWGSL};
@fragment fn fs() -> @location(0) vec4f {
  _ = textureDimensions(t, ${levelWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(levelArgType, Type.i32) || isConvertible(levelArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type,non_storage').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturedimensions').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
beginSubcases().
combine('textureType', keysOf(kValidTextureDimensionParameterTypesForNonStorageTextures)).
expand('hasLevelArg', (t) =>
kValidTextureDimensionParameterTypesForNonStorageTextures[t.textureType].hasLevelArg ?
[false, true] :
[false]
)
).
fn((t) => {
  const { testTextureType, hasLevelArg } = t.params;

  const levelWGSL = hasLevelArg ? ', 0' : '';

  const code = `
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureDimensions(t${levelWGSL});
  return vec4f(0);
}
`;

  const [baseTestTextureType] = getSampleAndBaseTextureTypeForTextureType(testTextureType);

  let expectSuccess = true;
  const types =
  kValidTextureDimensionParameterTypesForNonStorageTextures[baseTestTextureType] ||
  kValidTextureDimensionParameterTypesForStorageTextures[baseTestTextureType];
  if (types) {
    const typesMatch = !hasLevelArg || !!types.hasLevelArg;
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
      ${t.params.use ? '_ =' : ''} textureDimensions(t);
    }`;
  t.expectCompileResult(t.params.use, code);
});