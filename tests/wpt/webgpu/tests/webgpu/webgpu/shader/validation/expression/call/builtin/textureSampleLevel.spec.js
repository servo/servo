/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const builtin = 'textureSampleLevel';export const description = `
Validation tests for the ${builtin}() builtin.

* test textureSampleLevel coords parameter must be correct type
* test textureSampleLevel array_index parameter must be correct type
* test textureSampleLevel level parameter must be correct type
* test textureSampleLevel offset parameter must be correct type
* test textureSampleLevel offset parameter must be a const-expression
* test textureSampleLevel offset parameter must be between -8 and +7 inclusive
* test textureSampleLevel returns the correct type
* test textureSampleLevel doesn't work with texture types it's not supposed to
`;
import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf, objectsToRecord } from '../../../../../../common/util/data_tables.js';
import {
  Type,
  kAllScalarsAndVectors,
  isConvertible,


  isUnsignedType } from
'../../../../../util/conversion.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

import { kTestTextureTypes } from './shader_builtin_utils.js';








const kValidTextureSampleLevelParameterTypes = {
  'texture_2d<f32>': { coordsArgType: Type.vec2f, levelIsF32: true, offsetArgType: Type.vec2i },
  'texture_2d_array<f32>': {
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    levelIsF32: true,
    offsetArgType: Type.vec2i
  },
  'texture_3d<f32>': { coordsArgType: Type.vec3f, levelIsF32: true, offsetArgType: Type.vec3i },
  'texture_cube<f32>': { coordsArgType: Type.vec3f, levelIsF32: true },
  'texture_cube_array<f32>': {
    coordsArgType: Type.vec3f,
    hasArrayIndexArg: true,
    levelIsF32: true
  },
  texture_depth_2d: { coordsArgType: Type.vec2f, offsetArgType: Type.vec2i },
  texture_depth_2d_array: {
    coordsArgType: Type.vec2f,
    hasArrayIndexArg: true,
    offsetArgType: Type.vec2i
  },
  texture_depth_cube: { coordsArgType: Type.vec3f },
  texture_depth_cube_array: { coordsArgType: Type.vec3f, hasArrayIndexArg: true }
};

const kTextureTypes = keysOf(kValidTextureSampleLevelParameterTypes);
const kValuesTypes = objectsToRecord(kAllScalarsAndVectors);

export const g = makeTestGroup(ShaderValidationTest);

g.test('return_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates the return type of ${builtin} is the expected type.
`
).
params((u) =>
u.
combine('returnType', keysOf(kValuesTypes)).
combine('textureType', keysOf(kValidTextureSampleLevelParameterTypes)).
beginSubcases().
expand('offset', (t) =>
kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { returnType, textureType, offset } = t.params;
  const returnVarType = kValuesTypes[returnType];
  const { offsetArgType, coordsArgType, hasArrayIndexArg } =
  kValidTextureSampleLevelParameterTypes[textureType];
  const returnExpectedType = textureType.includes('depth') ? Type.f32 : Type.vec4f;

  const varWGSL = returnVarType.toString();
  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v: ${varWGSL} = textureSampleLevel(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(returnExpectedType, returnVarType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('coords_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates that only incorrect coords arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', keysOf(kValidTextureSampleLevelParameterTypes)).
combine('coordType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.coordType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { textureType, coordType, offset, value } = t.params;
  const coordArgType = kValuesTypes[coordType];
  const {
    offsetArgType,
    coordsArgType: coordsRequiredType,
    hasArrayIndexArg
  } = kValidTextureSampleLevelParameterTypes[textureType];

  const coordWGSL = coordArgType.create(value).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleLevel(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = isConvertible(coordArgType, coordsRequiredType);
  t.expectCompileResult(expectSuccess, code);
});

g.test('array_index_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates that only incorrect array_index arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no array_index
.filter((t) => !!kValidTextureSampleLevelParameterTypes[t.textureType].hasArrayIndexArg).
combine('arrayIndexType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.arrayIndexType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { textureType, arrayIndexType, value, offset } = t.params;
  const arrayIndexArgType = kValuesTypes[arrayIndexType];
  const args = [arrayIndexArgType.create(value)];
  const { coordsArgType, offsetArgType } = kValidTextureSampleLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleLevel(t, s, ${coordWGSL}, ${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(arrayIndexArgType, Type.i32) || isConvertible(arrayIndexArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('level_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates that only incorrect level arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('levelType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-1, 0, 1])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.levelType]) || t.value >= 0).
expand('offset', (t) =>
kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { textureType, levelType, value, offset } = t.params;
  const levelArgType = kValuesTypes[levelType];
  const args = [levelArgType.create(value)];
  const { coordsArgType, hasArrayIndexArg, offsetArgType, levelIsF32 } =
  kValidTextureSampleLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const levelWGSL = args.map((arg) => arg.wgsl()).join(', ');
  const offsetWGSL = offset ? `, ${offsetArgType.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleLevel(t, s, ${coordWGSL}${arrayWGSL}, ${levelWGSL}${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = levelIsF32 ?
  isConvertible(levelArgType, Type.f32) :
  isConvertible(levelArgType, Type.i32) || isConvertible(levelArgType, Type.u32);
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates that only incorrect offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes)
// filter out types with no offset
.filter((t) => !!kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType).
combine('offsetType', keysOf(kValuesTypes)).
beginSubcases().
combine('value', [-9, -8, 0, 7, 8])
// filter out unsigned types with negative values
.filter((t) => !isUnsignedType(kValuesTypes[t.offsetType]) || t.value >= 0)
).
fn((t) => {
  const { textureType, offsetType, value } = t.params;
  const offsetArgType = kValuesTypes[offsetType];
  const args = [offsetArgType.create(value)];
  const {
    coordsArgType,
    hasArrayIndexArg,
    offsetArgType: offsetRequiredType
  } = kValidTextureSampleLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = args.map((arg) => arg.wgsl()).join(', ');

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleLevel(t, s, ${coordWGSL}${arrayWGSL}, 0, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess =
  isConvertible(offsetArgType, offsetRequiredType) && value >= -8 && value <= 7;
  t.expectCompileResult(expectSuccess, code);
});

g.test('offset_argument,non_const').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates that only non-const offset arguments are rejected by ${builtin}
`
).
params((u) =>
u.
combine('textureType', kTextureTypes).
combine('varType', ['c', 'u', 'l'])
// filter out types with no offset
.filter((t) => !!kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType)
).
fn((t) => {
  const { textureType, varType } = t.params;
  const { coordsArgType, hasArrayIndexArg, offsetArgType } =
  kValidTextureSampleLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = `${offsetArgType}(${varType})`;

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${textureType};
@group(0) @binding(2) var<uniform> u: ${offsetArgType};
@fragment fn fs() -> @location(0) vec4f {
  const c = 1;
  let l = ${offsetArgType.create(0).wgsl()};
  let v = textureSampleLevel(t, s, ${coordWGSL}${arrayWGSL}, 0, ${offsetWGSL});
  return vec4f(0);
}
`;
  const expectSuccess = varType === 'c';
  t.expectCompileResult(expectSuccess, code);
});

g.test('texture_type').
specURL('https://gpuweb.github.io/gpuweb/wgsl/#texturesamplelevel').
desc(
  `
Validates that incompatible texture types don't work with ${builtin}
`
).
params((u) =>
u.
combine('testTextureType', kTestTextureTypes).
beginSubcases().
combine('textureType', keysOf(kValidTextureSampleLevelParameterTypes)).
expand('offset', (t) =>
kValidTextureSampleLevelParameterTypes[t.textureType].offsetArgType ?
[false, true] :
[false]
)
).
fn((t) => {
  const { testTextureType, textureType, offset } = t.params;
  const { coordsArgType, offsetArgType, hasArrayIndexArg } =
  kValidTextureSampleLevelParameterTypes[textureType];

  const coordWGSL = coordsArgType.create(0).wgsl();
  const arrayWGSL = hasArrayIndexArg ? ', 0' : '';
  const offsetWGSL = offset ? `, ${offsetArgType?.create(0).wgsl()}` : '';

  const code = `
@group(0) @binding(0) var s: sampler;
@group(0) @binding(1) var t: ${testTextureType};
@fragment fn fs() -> @location(0) vec4f {
  let v = textureSampleLevel(t, s, ${coordWGSL}${arrayWGSL}, 0${offsetWGSL});
  return vec4f(0);
}
`;

  const types = kValidTextureSampleLevelParameterTypes[testTextureType];
  const typesMatch = types ?
  types.coordsArgType === coordsArgType &&
  types.hasArrayIndexArg === hasArrayIndexArg && (
  offset ? types.offsetArgType === offsetArgType : true) :
  false;

  const expectSuccess = testTextureType === textureType || typesMatch;
  t.expectCompileResult(expectSuccess, code);
});

g.test('must_use').
desc('Tests that the result must be used').
params((u) => u.combine('use', [true, false])).
fn((t) => {
  const code = `
    @group(0) @binding(0) var t : texture_2d<f32>;
    @group(0) @binding(1) var s : sampler;
    fn foo() {
      ${t.params.use ? '_ =' : ''} textureSampleLevel(t,s,vec2(0,0), 0);
    }`;
  t.expectCompileResult(t.params.use, code);
});